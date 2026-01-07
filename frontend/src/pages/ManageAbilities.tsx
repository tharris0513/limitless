import React, { useState, useEffect } from 'react';
import { ArrowLeft, Plus, Edit, Trash2 } from 'lucide-react';
import GameAPI from '../services/api';
import type { Ability, PassiveEffect } from '../types/game';
import styles from './ManageClasses.module.css';

interface ManageAbilitiesProps {
  onBack: () => void;
  onLogout?: () => void;
}

export const ManageAbilities: React.FC<ManageAbilitiesProps> = ({ onBack }) => {
  const [abilities, setAbilities] = useState<Ability[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [editingAbility, setEditingAbility] = useState<Ability | null>(null);
  const [showAbilityForm, setShowAbilityForm] = useState(false);

  useEffect(() => {
    loadAbilities();
  }, []);

  const loadAbilities = async () => {
    try {
      setLoading(true);
      setError('');
      const abilitiesData = await GameAPI.adminGetAllAbilities();
      setAbilities(abilitiesData);
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to load abilities');
    } finally {
      setLoading(false);
    }
  };

  const handleSaveAbility = async (formData: {
    id?: string;
    name: string;
    description: string;
    abilityType: 'active' | 'passive';
    manaCost: number;
    cooldown: number;
    damageFormula?: string;
    healFormula?: string;
    effectFormula?: string;
    passiveEffect?: string;
  }) => {
    try {
      setLoading(true);
      setError('');

      const abilityData: Ability = {
        id: formData.id || formData.name.toLowerCase().replace(/\s+/g, '_'),
        name: formData.name,
        description: formData.description,
        abilityType: formData.abilityType,
        manaCost: formData.manaCost,
        cooldown: formData.cooldown,
        damageFormula: formData.damageFormula || undefined,
        healFormula: formData.healFormula || undefined,
        effectFormula: formData.effectFormula || undefined,
        passiveEffect: formData.passiveEffect || undefined,
      };

      if (editingAbility) {
        await GameAPI.adminUpdateAbility(editingAbility.id, abilityData);
      } else {
        await GameAPI.adminCreateAbility(abilityData);
      }

      await loadAbilities();
      setShowAbilityForm(false);
      setEditingAbility(null);
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to save ability');
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteAbility = async (abilityId: string) => {
    if (!confirm('Are you sure you want to delete this ability?')) return;

    try {
      setLoading(true);
      await GameAPI.adminDeleteAbility(abilityId);
      await loadAbilities();
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to delete ability');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onBack}>
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
        <h1 className={styles.title}>⚔️ Manage Abilities</h1>
      </div>

      {error && <div className={styles.error}>{error}</div>}

      <div className={styles.content}>
        <button
          className={styles.addButton}
          onClick={() => {
            setEditingAbility(null);
            setShowAbilityForm(true);
          }}
        >
          <Plus size={16} /> New Ability
        </button>

        {loading && <div className={styles.loading}>Loading...</div>}

        <div className={styles.list}>
          {abilities.map(ability => (
            <div key={ability.id} className={styles.card}>
              <div className={styles.cardHeader}>
                <h3>{ability.name}</h3>
                <div className={styles.actions}>
                  <button onClick={() => { setEditingAbility(ability); setShowAbilityForm(true); }}>
                    <Edit size={16} />
                  </button>
                  <button onClick={() => handleDeleteAbility(ability.id)}>
                    <Trash2 size={16} />
                  </button>
                </div>
              </div>
              <p className={styles.description}>{ability.description}</p>
              <div className={styles.abilityInfo}>
                <span>Type: {ability.abilityType}</span>
                <span>Mana: {ability.manaCost}</span>
                <span>Cooldown: {ability.cooldown}</span>
              </div>
              {ability.abilityType === 'passive' && ability.passiveEffect && (
                <div className={styles.formulaDisplay}>
                  <strong>Passive Effect:</strong> {ability.passiveEffect}
                </div>
              )}
              {ability.abilityType === 'active' && (
                <>
                  {ability.damageFormula && (
                    <div className={styles.formulaDisplay}>
                      <strong>Damage:</strong> {ability.damageFormula}
                    </div>
                  )}
                  {ability.healFormula && (
                    <div className={styles.formulaDisplay}>
                      <strong>Heal:</strong> {ability.healFormula}
                    </div>
                  )}
                  {ability.effectFormula && (
                    <div className={styles.formulaDisplay}>
                      <strong>Effect:</strong> {ability.effectFormula}
                    </div>
                  )}
                </>
              )}
            </div>
          ))}
        </div>
      </div>

      {showAbilityForm && (
        <AbilityForm
          initialData={editingAbility}
          onSave={handleSaveAbility}
          onCancel={() => { setShowAbilityForm(false); setEditingAbility(null); }}
        />
      )}
    </div>
  );
};

// Ability Form Component
const AbilityForm: React.FC<{
  initialData: Ability | null;
  onSave: (data: {
    id?: string;
    name: string;
    description: string;
    abilityType: 'active' | 'passive';
    manaCost: number;
    cooldown: number;
    damageFormula?: string;
    healFormula?: string;
    effectFormula?: string;
    passiveEffect?: string;
  }) => void;
  onCancel: () => void;
}> = ({ initialData, onSave, onCancel }) => {
  const [formData, setFormData] = useState(
    initialData ? {
      ...initialData,
      damageFormula: initialData.damageFormula || '',
      healFormula: initialData.healFormula || '',
      effectFormula: initialData.effectFormula || '',
      passiveEffect: initialData.passiveEffect || '',
    } : {
      name: '',
      description: '',
      abilityType: 'active' as 'active' | 'passive',
      manaCost: 0,
      cooldown: 0,
      damageFormula: '',
      healFormula: '',
      effectFormula: '',
      passiveEffect: '',
    }
  );

  const [passiveEffects, setPassiveEffects] = useState<PassiveEffect[]>([]);
  const [loadingEffects, setLoadingEffects] = useState(false);

  useEffect(() => {
    if (formData.abilityType === 'passive') {
      loadPassiveEffects();
    }
  }, [formData.abilityType]);

  const loadPassiveEffects = async () => {
    try {
      setLoadingEffects(true);
      const effects = await GameAPI.adminGetPassiveEffects();
      setPassiveEffects(effects);
    } catch (err) {
      console.error('Failed to load passive effects:', err);
    } finally {
      setLoadingEffects(false);
    }
  };

  return (
    <div className={styles.modal}>
      <div className={styles.modalContent}>
        <h2>{initialData ? 'Edit Ability' : 'New Ability'}</h2>
        <form onSubmit={(e) => { e.preventDefault(); onSave(formData); }}>
          <input
            type="text"
            placeholder="Ability Name"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            required
          />
          <textarea
            placeholder="Description"
            value={formData.description}
            onChange={(e) => setFormData({ ...formData, description: e.target.value })}
            required
          />
          <select value={formData.abilityType} onChange={(e) => setFormData({ ...formData, abilityType: e.target.value as 'active' | 'passive' })}>
            <option value="active">Active</option>
            <option value="passive">Passive</option>
          </select>
          
          {formData.abilityType === 'passive' ? (
            <div className={styles.formulaSection}>
              <h3>Passive Effect</h3>
              <p className={styles.formulaHelp}>Select the passive effect this ability provides</p>
              <label>
                <span>Effect Type</span>
                {loadingEffects ? (
                  <p>Loading effects...</p>
                ) : (
                  <select 
                    value={formData.passiveEffect}
                    onChange={(e) => setFormData({ ...formData, passiveEffect: e.target.value })}
                    required
                  >
                    <option value="">-- Select a passive effect --</option>
                    {passiveEffects.map(effect => (
                      <option key={effect.id} value={effect.id}>
                        {effect.description}
                      </option>
                    ))}
                  </select>
                )}
              </label>
            </div>
          ) : (
            <div className={styles.formulaSection}>
              <h3>Formulas (Advanced)</h3>
              <p className={styles.formulaHelp}>Available variables: might, defense, magic, resistance, agility, level</p>
              <label>
                <span>Damage Formula</span>
                <input 
                  type="text" 
                  placeholder="e.g., (might * 0.8) + 15" 
                  value={formData.damageFormula}
                  onChange={(e) => setFormData({ ...formData, damageFormula: e.target.value })}
                />
              </label>
              <label>
                <span>Heal Formula</span>
                <input 
                  type="text" 
                  placeholder="e.g., (magic * 1.2) + 20" 
                  value={formData.healFormula}
                  onChange={(e) => setFormData({ ...formData, healFormula: e.target.value })}
                />
              </label>
              <label>
                <span>Effect Formula</span>
                <input 
                  type="text" 
                  placeholder="e.g., magic * 0.5" 
                  value={formData.effectFormula}
                  onChange={(e) => setFormData({ ...formData, effectFormula: e.target.value })}
                />
              </label>
            </div>
          )}

          {formData.abilityType === 'active' && (
            <div className={styles.formGrid}>
              <label>
                <span>Mana Cost</span>
                <input type="number" placeholder="Mana Cost" value={formData.manaCost} onChange={(e) => setFormData({ ...formData, manaCost: parseInt(e.target.value) || 0 })} />
              </label>
              <label>
                <span>Cooldown</span>
                <input type="number" placeholder="Cooldown" value={formData.cooldown} onChange={(e) => setFormData({ ...formData, cooldown: parseInt(e.target.value) || 0 })} />
              </label>
            </div>
          )}
          <div className={styles.formActions}>
            <button type="submit" className={styles.saveButton}>Save</button>
            <button type="button" onClick={onCancel} className={styles.cancelButton}>Cancel</button>
          </div>
        </form>
      </div>
    </div>
  );
};
