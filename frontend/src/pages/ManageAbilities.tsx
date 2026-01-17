import React, { useState, useEffect } from 'react';
import { ArrowLeft, Plus, Edit, Trash2, X } from 'lucide-react';
import GameAPI from '../services/api';
import type { Ability, AbilityEffect, PassiveEffect } from '../types/game';
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

  const handleSaveAbility = async (abilityData: Ability) => {
    try {
      setLoading(true);
      setError('');

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
          {abilities
            .sort((a, b) => a.name.localeCompare(b.name))
            .map(ability => (
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
                <span>Effects: {ability.effects?.length || 0}</span>
                <span>Mana: {ability.manaCost}</span>
                <span>Cooldown: {ability.cooldown}</span>
              </div>
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

// Ability Form Component with Multi-Effect Support
const AbilityForm: React.FC<{
  initialData: Ability | null;
  onSave: (data: Ability) => void;
  onCancel: () => void;
}> = ({ initialData, onSave, onCancel }) => {
  const [formData, setFormData] = useState<{
    name: string;
    description: string;
    abilityType: string;
    manaCost: number;
    cooldown: number;
    effects: AbilityEffect[];
  }>(
    initialData ? {
      name: initialData.name,
      description: initialData.description,
      abilityType: initialData.abilityType || 'combat',
      manaCost: initialData.manaCost || 0,
      cooldown: initialData.cooldown || 0,
      effects: initialData.effects || [],
    } : {
      name: '',
      description: '',
      abilityType: 'combat',
      manaCost: 0,
      cooldown: 0,
      effects: [],
    }
  );

  const [passiveEffects, setPassiveEffects] = useState<PassiveEffect[]>([]);
  const [loadingEffects, setLoadingEffects] = useState(false);

  useEffect(() => {
    loadPassiveEffects();
  }, []);

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

  const addEffect = () => {
    const newEffect: AbilityEffect = {
      id: `effect_${Date.now()}`,
      effectType: 'active',
      activeType: 'damage',
      formula: '',
      attackDescription: '',
    };
    setFormData({
      ...formData,
      effects: [...formData.effects, newEffect],
    });
  };

  const removeEffect = (effectId: string) => {
    setFormData({
      ...formData,
      effects: formData.effects.filter(e => e.id !== effectId),
    });
  };

  const updateEffect = (effectId: string, updates: Partial<AbilityEffect>) => {
    setFormData({
      ...formData,
      effects: formData.effects.map(e => 
        e.id === effectId ? { ...e, ...updates } : e
      ),
    });
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    const abilityData: Ability = {
      id: initialData?.id || formData.name.toLowerCase().replace(/\s+/g, '_'),
      name: formData.name,
      description: formData.description,
      abilityType: formData.abilityType as 'combat' | 'passive',
      manaCost: formData.abilityType === 'passive' ? 0 : formData.manaCost,
      cooldown: formData.abilityType === 'passive' ? 0 : formData.cooldown,
      effects: formData.effects,
    };
    
    onSave(abilityData);
  };

  return (
    <div className={styles.modal}>
      <div className={styles.modalContent} style={{ maxWidth: '800px' }}>
        <h2>{initialData ? 'Edit Ability' : 'New Ability'}</h2>
        <form onSubmit={handleSubmit}>
          <div className={styles.formGrid}>
            <label style={{ gridColumn: '1 / -1' }}>
              <span>Ability Name *</span>
              <input
                type="text"
                placeholder="Ability Name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                required
              />
            </label>
            
            <label style={{ gridColumn: '1 / -1' }}>
              <span>Description *</span>
              <textarea
                placeholder="Description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                required
                rows={3}
              />
            </label>

            <label style={{ gridColumn: '1 / -1' }}>
              <span>Ability Type *</span>
              <select
                value={formData.abilityType}
                onChange={(e) => setFormData({ ...formData, abilityType: e.target.value })}
                required
              >
                <option value="combat">Combat - Shows in action bar</option>
                <option value="passive">Passive - Automatic effect</option>
              </select>
            </label>

            {formData.abilityType === 'combat' && (
              <>
                <label>
                  <span>Mana Cost</span>
                  <input 
                    type="number" 
                    value={formData.manaCost} 
                    onChange={(e) => setFormData({ ...formData, manaCost: parseInt(e.target.value) || 0 })} 
                  />
                </label>
                
                <label>
                  <span>Cooldown (turns)</span>
                  <input 
                    type="number" 
                    value={formData.cooldown} 
                    onChange={(e) => setFormData({ ...formData, cooldown: parseInt(e.target.value) || 0 })} 
                  />
                </label>
              </>
            )}
          </div>

          <div style={{ marginTop: '24px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
              <h3 style={{ margin: 0 }}>Effects</h3>
              <button 
                type="button" 
                className={styles.addButton}
                onClick={addEffect}
                style={{ padding: '8px 16px', fontSize: '14px' }}
              >
                <Plus size={16} /> Add Effect
              </button>
            </div>

            {formData.effects.length === 0 && (
              <div style={{ 
                padding: '24px', 
                textAlign: 'center', 
                color: '#888',
                border: '2px dashed #333',
                borderRadius: '8px',
                marginBottom: '16px'
              }}>
                No effects added yet. Click "Add Effect" to create one.
              </div>
            )}

            {formData.effects.map((effect, index) => (
              <EffectEditor
                key={effect.id}
                effect={effect}
                index={index}
                abilityType={formData.abilityType}
                passiveEffects={passiveEffects}
                loadingEffects={loadingEffects}
                onUpdate={(updates) => updateEffect(effect.id, updates)}
                onRemove={() => removeEffect(effect.id)}
              />
            ))}
          </div>

          <div className={styles.formActions} style={{ marginTop: '24px' }}>
            <button type="submit" className={styles.saveButton}>
              {initialData ? 'Update Ability' : 'Create Ability'}
            </button>
            <button type="button" onClick={onCancel} className={styles.cancelButton}>
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

// Effect Editor Component
const EffectEditor: React.FC<{
  effect: AbilityEffect;
  index: number;
  abilityType: string;
  passiveEffects: PassiveEffect[];
  loadingEffects: boolean;
  onUpdate: (updates: Partial<AbilityEffect>) => void;
  onRemove: () => void;
}> = ({ effect, index, abilityType, passiveEffects, loadingEffects, onUpdate, onRemove }) => {
  return (
    <div style={{
      border: '1px solid #333',
      borderRadius: '8px',
      padding: '16px',
      marginBottom: '16px',
      backgroundColor: '#0a0a0a'
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
        <h4 style={{ margin: 0 }}>Effect #{index + 1}</h4>
        <button
          type="button"
          onClick={onRemove}
          style={{
            background: '#dc2626',
            border: 'none',
            color: 'white',
            padding: '6px 12px',
            borderRadius: '4px',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: '4px'
          }}
        >
          <X size={14} /> Remove
        </button>
      </div>

      <div className={styles.formGrid}>
        <label>
          <span>Effect Type *</span>
          <select
            value={effect.effectType}
            onChange={(e) => {
              const newType = e.target.value as 'active' | 'passive';
              onUpdate({ 
                effectType: newType,
                // Reset type-specific fields
                activeType: newType === 'active' ? 'damage' : undefined,
                passiveType: newType === 'passive' ? '' : undefined,
                formula: newType === 'active' ? '' : undefined,
                attackDescription: newType === 'active' ? '' : undefined,
              });
            }}
            required
          >
            {abilityType !== 'passive' && <option value="active">Active</option>}
            <option value="passive">Passive</option>
          </select>
        </label>

        {effect.effectType === 'active' ? (
          <>
            <label>
              <span>Active Type *</span>
              <select
                value={effect.activeType || 'damage'}
                onChange={(e) => onUpdate({ activeType: e.target.value as 'damage' | 'heal' })}
                required
              >
                <option value="damage">Damage</option>
                <option value="heal">Heal</option>
              </select>
            </label>

            <label style={{ gridColumn: '1 / -1' }}>
              <span>Formula *</span>
              <input
                type="text"
                placeholder="e.g., (might * 1.2) + (magic * 0.5) + 10"
                value={effect.formula || ''}
                onChange={(e) => onUpdate({ formula: e.target.value })}
                required
              />
              <small style={{ color: '#888', fontSize: '0.85em', marginTop: '4px', display: 'block' }}>
                Available variables: might, defense, magic, resistance, agility, level
              </small>
            </label>

            <label style={{ gridColumn: '1 / -1' }}>
              <span>Attack Description (Optional)</span>
              <input
                type="text"
                placeholder="e.g., You unleash flames for ${damage} damage!"
                value={effect.attackDescription || ''}
                onChange={(e) => onUpdate({ attackDescription: e.target.value })}
              />
              <small style={{ color: '#888', fontSize: '0.85em', marginTop: '4px', display: 'block' }}>
                Use {'${damage}'} or {'${x}'} for damage, {'${name}'} for ability name
              </small>
            </label>
          </>
        ) : (
          <label style={{ gridColumn: '1 / -1' }}>
            <span>Passive Type *</span>
            {loadingEffects ? (
              <p>Loading effects...</p>
            ) : (
              <select
                value={effect.passiveType || ''}
                onChange={(e) => onUpdate({ passiveType: e.target.value })}
                required
              >
                <option value="">-- Select a passive effect --</option>
                {passiveEffects.map(pe => (
                  <option key={pe.id} value={pe.id}>
                    {pe.description}
                  </option>
                ))}
              </select>
            )}
          </label>
        )}
      </div>
    </div>
  );
};
