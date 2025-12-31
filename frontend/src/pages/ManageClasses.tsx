import React, { useState, useEffect, useRef } from 'react';
import { ArrowLeft, Plus, Edit, Trash2, Download, Upload } from 'lucide-react';
import GameAPI from '../services/api';
import type { Class, Ability } from '../types/game';
import styles from './ManageClasses.module.css';

interface ManageClassesProps {
  onBack: () => void;
  onLogout?: () => void;
  onAdminClick?: () => void;
}

export const ManageClasses: React.FC<ManageClassesProps> = ({ onBack }) => {
  const [classes, setClasses] = useState<Class[]>([]);
  const [abilities, setAbilities] = useState<Ability[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [activeTab, setActiveTab] = useState<'classes' | 'abilities'>('classes');
  const [editingClass, setEditingClass] = useState<Class | null>(null);
  const [editingAbility, setEditingAbility] = useState<Ability | null>(null);
  const [showClassForm, setShowClassForm] = useState(false);
  const [showAbilityForm, setShowAbilityForm] = useState(false);
  const [showAbilityAssignment, setShowAbilityAssignment] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      setLoading(true);
      setError('');
      const [classesData, abilitiesData] = await Promise.all([
        GameAPI.getClasses(),
        GameAPI.adminGetAllAbilities(),
      ]);
      setClasses(classesData);
      setAbilities(abilitiesData);
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const handleSaveClass = async (formData: {
    id?: string;
    name: string;
    description: string;
    startingMight: number;
    startingDefense: number;
    startingMagic: number;
    startingResistance: number;
    startingAgility: number;
    startingHealth: number;
    startingMana: number;
  }) => {
    try {
      setLoading(true);
      setError('');
      
      const classData: Class = {
        id: formData.id || formData.name.toLowerCase().replace(/\s+/g, '_'),
        name: formData.name,
        description: formData.description,
        startingMight: formData.startingMight,
        startingDefense: formData.startingDefense,
        startingMagic: formData.startingMagic,
        startingResistance: formData.startingResistance,
        startingAgility: formData.startingAgility,
        startingHealth: formData.startingHealth,
        startingMana: formData.startingMana,
      };

      if (editingClass) {
        await GameAPI.adminUpdateClass(editingClass.id, classData);
      } else {
        await GameAPI.adminCreateClass(classData);
      }

      await loadData();
      setShowClassForm(false);
      setEditingClass(null);
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to save class');
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteClass = async (classId: string) => {
    if (!confirm('Are you sure you want to delete this class?')) return;
    
    try {
      setLoading(true);
      await GameAPI.adminDeleteClass(classId);
      await loadData();
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to delete class');
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
      };

      if (editingAbility) {
        await GameAPI.adminUpdateAbility(editingAbility.id, abilityData);
      } else {
        await GameAPI.adminCreateAbility(abilityData);
      }

      await loadData();
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
      await loadData();
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to delete ability');
    } finally {
      setLoading(false);
    }
  };

  const handleAddAbilityToClass = async (classId: string, abilityId: string, unlockLevel: number) => {
    try {
      setLoading(true);
      await GameAPI.adminAddClassAbility(classId, abilityId, unlockLevel);
      await loadData();
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to add ability to class');
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveAbilityFromClass = async (classId: string, abilityId: string) => {
    try {
      setLoading(true);
      await GameAPI.adminRemoveClassAbility(classId, abilityId);
      await loadData();
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to remove ability from class');
    } finally {
      setLoading(false);
    }
  };

  const handleExportConfig = async () => {
    try {
      setLoading(true);
      const config = await GameAPI.adminExportConfig();
      
      // Create download link
      const dataStr = JSON.stringify(config, null, 2);
      const dataBlob = new Blob([dataStr], { type: 'application/json' });
      const url = URL.createObjectURL(dataBlob);
      const link = document.createElement('a');
      link.href = url;
      link.download = `game-config-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to export configuration');
    } finally {
      setLoading(false);
    }
  };

  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleImportConfig = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    try {
      setLoading(true);
      const text = await file.text();
      const config = JSON.parse(text);
      
      if (!confirm(`Import ${config.classes?.length || 0} classes and ${config.abilities?.length || 0} abilities? This will overwrite existing data.`)) {
        return;
      }

      await GameAPI.adminImportConfig(config);
      await loadData();
      alert('Configuration imported successfully!');
    } catch (err: unknown) {
      const error = err as { response?: { data?: { error?: string } } };
      setError(error?.response?.data?.error || 'Failed to import configuration');
    } finally {
      setLoading(false);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    }
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onBack}>
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
        <h1 className={styles.title}>⚔️ Manage Classes & Abilities</h1>
        <div className={styles.exportImportButtons}>
          <button className={styles.exportButton} onClick={handleExportConfig}>
            <Download size={16} /> Export
          </button>
          <input
            ref={fileInputRef}
            type="file"
            accept=".json"
            style={{ display: 'none' }}
            onChange={handleImportConfig}
          />
          <button 
            className={styles.importButton} 
            onClick={() => fileInputRef.current?.click()}
          >
            <Upload size={16} /> Import
          </button>
        </div>
      </div>

      {error && <div className={styles.error}>{error}</div>}

      <div className={styles.tabs}>
        <button
          className={`${styles.tab} ${activeTab === 'classes' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('classes')}
        >
          Classes
        </button>
        <button
          className={`${styles.tab} ${activeTab === 'abilities' ? styles.activeTab : ''}`}
          onClick={() => setActiveTab('abilities')}
        >
          Abilities
        </button>
      </div>

      {activeTab === 'classes' && (
        <div className={styles.content}>
          <button
            className={styles.addButton}
            onClick={() => {
              setEditingClass(null);
              setShowClassForm(true);
            }}
          >
            <Plus size={16} /> New Class
          </button>

          {loading && <div className={styles.loading}>Loading...</div>}

          <div className={styles.list}>
            {classes.map(cls => (
              <div key={cls.id} className={styles.card}>
                <div className={styles.cardHeader}>
                  <h3>{cls.name}</h3>
                  <div className={styles.actions}>
                    <button onClick={() => { setEditingClass(cls); setShowClassForm(true); }}>
                      <Edit size={16} />
                    </button>
                    <button onClick={() => handleDeleteClass(cls.id)}>
                      <Trash2 size={16} />
                    </button>
                  </div>
                </div>
                <p className={styles.description}>{cls.description}</p>
                <div className={styles.stats}>
                  <span>⚔️ {cls.startingMight}</span>
                  <span>🛡️ {cls.startingDefense}</span>
                  <span>🧙 {cls.startingMagic}</span>
                  <span>🔮 {cls.startingResistance}</span>
                  <span>💨 {cls.startingAgility}</span>
                  <span>❤️ {cls.startingHealth}</span>
                  <span>⚡ {cls.startingMana}</span>
                </div>
                <button
                  className={styles.manageAbilitiesButton}
                  onClick={() => setShowAbilityAssignment(cls.id)}
                >
                  Manage Abilities
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {activeTab === 'abilities' && (
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
              </div>
            ))}
          </div>
        </div>
      )}

      {showClassForm && (
        <ClassForm
          initialData={editingClass}
          onSave={handleSaveClass}
          onCancel={() => { setShowClassForm(false); setEditingClass(null); }}
        />
      )}

      {showAbilityForm && (
        <AbilityForm
          initialData={editingAbility}
          onSave={handleSaveAbility}
          onCancel={() => { setShowAbilityForm(false); setEditingAbility(null); }}
        />
      )}

      {showAbilityAssignment !== null && (
        <AbilityAssignment
          classId={showAbilityAssignment}
          availableAbilities={abilities}
          onAdd={handleAddAbilityToClass}
          onRemove={handleRemoveAbilityFromClass}
          onClose={() => setShowAbilityAssignment(null)}
        />
      )}
    </div>
  );
};

// Class Form Component
const ClassForm: React.FC<{
  initialData: Class | null;
  onSave: (data: {
    id?: string;
    name: string;
    description: string;
    startingMight: number;
    startingDefense: number;
    startingMagic: number;
    startingResistance: number;
    startingAgility: number;
    startingHealth: number;
    startingMana: number;
  }) => void;
  onCancel: () => void;
}> = ({ initialData, onSave, onCancel }) => {
  const [formData, setFormData] = useState(
    initialData || {
      name: '',
      description: '',
      startingMight: 10,
      startingDefense: 10,
      startingMagic: 10,
      startingResistance: 10,
      startingAgility: 10,
      startingHealth: 100,
      startingMana: 50,
    }
  );

  return (
    <div className={styles.modal}>
      <div className={styles.modalContent}>
        <h2>{initialData ? 'Edit Class' : 'New Class'}</h2>
        <form onSubmit={(e) => { e.preventDefault(); onSave(formData); }}>
          <label>
            <span>Class Name</span>
            <input
              type="text"
              placeholder="Class Name"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              required
            />
          </label>
          <label>
            <span>Description</span>
            <textarea
              placeholder="Description"
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              required
            />
          </label>
          <div className={styles.formGrid}>
            <label>
              <span>Might</span>
              <input type="number" placeholder="Might" value={formData.startingMight} onChange={(e) => setFormData({ ...formData, startingMight: parseInt(e.target.value) || 0 })} />
            </label>
            <label>
              <span>Defense</span>
              <input type="number" placeholder="Defense" value={formData.startingDefense} onChange={(e) => setFormData({ ...formData, startingDefense: parseInt(e.target.value) || 0 })} />
            </label>
            <label>
              <span>Magic</span>
              <input type="number" placeholder="Magic" value={formData.startingMagic} onChange={(e) => setFormData({ ...formData, startingMagic: parseInt(e.target.value) || 0 })} />
            </label>
            <label>
              <span>Resistance</span>
              <input type="number" placeholder="Resistance" value={formData.startingResistance} onChange={(e) => setFormData({ ...formData, startingResistance: parseInt(e.target.value) || 0 })} />
            </label>
            <label>
              <span>Agility</span>
              <input type="number" placeholder="Agility" value={formData.startingAgility} onChange={(e) => setFormData({ ...formData, startingAgility: parseInt(e.target.value) || 0 })} />
            </label>
            <label>
              <span>Starting Health</span>
              <input type="number" placeholder="Health" value={formData.startingHealth} onChange={(e) => setFormData({ ...formData, startingHealth: parseInt(e.target.value) || 0 })} />
            </label>
            <label>
              <span>Starting Mana</span>
              <input type="number" placeholder="Mana" value={formData.startingMana} onChange={(e) => setFormData({ ...formData, startingMana: parseInt(e.target.value) || 0 })} />
            </label>
          </div>
          <div className={styles.formActions}>
            <button type="submit" className={styles.saveButton}>Save</button>
            <button type="button" onClick={onCancel} className={styles.cancelButton}>Cancel</button>
          </div>
        </form>
      </div>
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
  }) => void;
  onCancel: () => void;
}> = ({ initialData, onSave, onCancel }) => {
  const [formData, setFormData] = useState(
    initialData ? {
      ...initialData,
      damageFormula: initialData.damageFormula || '',
      healFormula: initialData.healFormula || '',
      effectFormula: initialData.effectFormula || '',
    } : {
      name: '',
      description: '',
      abilityType: 'active' as 'active' | 'passive',
      manaCost: 0,
      cooldown: 0,
      damageFormula: '',
      healFormula: '',
      effectFormula: '',
    }
  );

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
          <div className={styles.formActions}>
            <button type="submit" className={styles.saveButton}>Save</button>
            <button type="button" onClick={onCancel} className={styles.cancelButton}>Cancel</button>
          </div>
        </form>
      </div>
    </div>
  );
};

// Ability Assignment Component
const AbilityAssignment: React.FC<{
  classId: string;
  availableAbilities: Ability[];
  onAdd: (classId: string, abilityId: string, unlockLevel: number) => void;
  onRemove?: (classId: string, abilityId: string) => void;
  onClose: () => void;
}> = ({ classId, availableAbilities, onAdd, onRemove, onClose }) => {
  const [selectedAbility, setSelectedAbility] = useState('');
  const [unlockLevel, setUnlockLevel] = useState(1);
  const [assignedAbilities, setAssignedAbilities] = useState<Array<{ ability: Ability; unlockLevel: number }>>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadAssignedAbilities();
  }, [classId]);

  const loadAssignedAbilities = async () => {
    try {
      setLoading(true);
      const abilities = await GameAPI.getClassAbilities(classId);
      setAssignedAbilities(abilities);
    } catch (err) {
      console.error('Failed to load class abilities:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleAdd = async () => {
    if (selectedAbility) {
      await onAdd(classId, selectedAbility, unlockLevel);
      setSelectedAbility('');
      setUnlockLevel(1);
      await loadAssignedAbilities();
    }
  };

  const handleRemove = async (abilityId: string) => {
    if (onRemove) {
      await onRemove(classId, abilityId);
      await loadAssignedAbilities();
    }
  };

  return (
    <div className={styles.modal}>
      <div className={styles.modalContent}>
        <h2>Manage Class Abilities</h2>
        
        {loading ? (
          <div className={styles.loading}>Loading...</div>
        ) : (
          <>
            {assignedAbilities.length > 0 && (
              <div style={{ marginBottom: '1.5rem' }}>
                <h3 style={{ color: '#00ff00', marginBottom: '0.5rem' }}>Current Abilities</h3>
                {assignedAbilities.map(({ ability, unlockLevel }) => (
                  <div key={ability.id} className={styles.card} style={{ marginBottom: '0.5rem' }}>
                    <div className={styles.cardHeader}>
                      <span>{ability.name} (Level {unlockLevel})</span>
                      <div className={styles.actions}>
                        <button onClick={() => handleRemove(ability.id)}>
                          <Trash2 size={16} />
                        </button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}

            <h3 style={{ color: '#00ff00', marginBottom: '0.5rem' }}>Add New Ability</h3>
            <div className={styles.abilityAssignment}>
          <label>
            <span>Ability</span>
            <select value={selectedAbility} onChange={(e) => setSelectedAbility(e.target.value)}>
              <option value="">Select Ability</option>
              {availableAbilities.map(ability => (
                <option key={ability.id} value={ability.id}>{ability.name}</option>
              ))}
            </select>
          </label>
          <label>
            <span>Unlock Level</span>
            <input
              type="number"
              min="1"
              placeholder="Unlock Level"
              value={unlockLevel}
              onChange={(e) => setUnlockLevel(parseInt(e.target.value))}
            />
          </label>
          <label>
            <span>&nbsp;</span>
            <button
              onClick={handleAdd}
              className={styles.saveButton}
            >
              Add
            </button>
          </label>
        </div>
        </>
        )}
        <div className={styles.formActions}>
          <button onClick={onClose} className={styles.cancelButton}>Close</button>
        </div>
      </div>
    </div>
  );
};
