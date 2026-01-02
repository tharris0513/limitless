import { useEffect, useState } from 'react';
import { ArrowLeft } from 'lucide-react';
import GameAPI from '../services/api';
import type { Adventure } from '../types/game';
import styles from './ManageAdventures.module.css';

interface ManageAdventuresProps {
  onBack: () => void;
  onLogout?: () => void;
}

export default function ManageAdventures({ onBack }: ManageAdventuresProps) {
  const [adventures, setAdventures] = useState<Adventure[]>([]);
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [formData, setFormData] = useState<Partial<Adventure>>({
    name: '',
    description: '',
    requiredLevel: 1,
    adventureType: 'exploration',
    experienceReward: 50,
  });

  useEffect(() => {
    fetchAdventures();
  }, []);

  const fetchAdventures = async () => {
    setLoading(true);
    try {
      const data = await GameAPI.adminGetAllAdventures();
      setAdventures(data);
    } catch (error) {
      console.error('Failed to fetch adventures:', error);
      alert('Failed to load adventures');
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await GameAPI.adminCreateAdventure(formData);
      setCreating(false);
      resetForm();
      fetchAdventures();
    } catch (error) {
      console.error('Failed to create adventure:', error);
      alert('Failed to create adventure');
    }
  };

  const handleUpdate = async (id: string) => {
    try {
      await GameAPI.adminUpdateAdventure(id, formData);
      setEditing(null);
      fetchAdventures();
    } catch (error) {
      console.error('Failed to update adventure:', error);
      alert('Failed to update adventure');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this adventure?')) return;
    
    try {
      await GameAPI.adminDeleteAdventure(id);
      fetchAdventures();
    } catch (error) {
      console.error('Failed to delete adventure:', error);
      alert('Failed to delete adventure');
    }
  };

  const startEdit = (adventure: Adventure) => {
    setEditing(adventure.id);
    setFormData(adventure);
  };

  const resetForm = () => {
    setFormData({
      name: '',
      description: '',
      requiredLevel: 1,
      adventureType: 'exploration',
      experienceReward: 50,
    });
  };

  const cancelEdit = () => {
    setEditing(null);
    resetForm();
  };

  if (loading) {
    return <div className={styles.container}>Loading adventures...</div>;
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onBack}>
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
        <h1>Adventure Management</h1>
        <button onClick={() => setCreating(true)} className={styles.createButton}>
          Create New Adventure
        </button>
      </div>

      {creating && (
        <div className={styles.modal}>
          <div className={styles.modalContent}>
            <h2>Create New Adventure</h2>
            <form onSubmit={handleCreate}>
              <div className={styles.formGrid}>
                <div className={styles.formGroup}>
                  <label>Name *</label>
                  <input
                    type="text"
                    value={formData.name}
                    onChange={e => setFormData({ ...formData, name: e.target.value })}
                    required
                  />
                </div>

                <div className={styles.formGroup}>
                  <label>Type *</label>
                  <select
                    value={formData.adventureType}
                    onChange={e => setFormData({ ...formData, adventureType: e.target.value })}
                    required
                  >
                    <option value="exploration">Exploration</option>
                    <option value="puzzle">Puzzle</option>
                    <option value="dialogue">Dialogue</option>
                    <option value="quest">Quest</option>
                    <option value="treasure">Treasure Hunt</option>
                    <option value="mystery">Mystery</option>
                  </select>
                </div>

                <div className={styles.formGroup}>
                  <label>Required Level *</label>
                  <input
                    type="number"
                    value={formData.requiredLevel}
                    onChange={e => setFormData({ ...formData, requiredLevel: parseInt(e.target.value) })}
                    min="1"
                    required
                  />
                </div>

                <div className={styles.formGroup}>
                  <label>Experience Reward *</label>
                  <input
                    type="number"
                    value={formData.experienceReward}
                    onChange={e => setFormData({ ...formData, experienceReward: parseInt(e.target.value) })}
                    min="0"
                    required
                  />
                </div>

                <div className={styles.formGroup} style={{ gridColumn: '1 / -1' }}>
                  <label>Description *</label>
                  <textarea
                    value={formData.description}
                    onChange={e => setFormData({ ...formData, description: e.target.value })}
                    rows={4}
                    required
                  />
                </div>
              </div>

              <div className={styles.formActions}>
                <button type="submit" className={styles.submitButton}>
                  Create Adventure
                </button>
                <button type="button" onClick={() => { setCreating(false); resetForm(); }} className={styles.cancelButton}>
                  Cancel
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      <div className={styles.adventureGrid}>
        {adventures.map(adventure => (
          <div key={adventure.id} className={styles.adventureCard}>
            {editing === adventure.id ? (
              <form onSubmit={e => { e.preventDefault(); handleUpdate(adventure.id); }}>
                <div className={styles.formGrid}>
                  <div className={styles.formGroup}>
                    <label>Name</label>
                    <input
                      type="text"
                      value={formData.name}
                      onChange={e => setFormData({ ...formData, name: e.target.value })}
                      required
                    />
                  </div>

                  <div className={styles.formGroup}>
                    <label>Type</label>
                    <select
                      value={formData.adventureType}
                      onChange={e => setFormData({ ...formData, adventureType: e.target.value })}
                      required
                    >
                      <option value="exploration">Exploration</option>
                      <option value="puzzle">Puzzle</option>
                      <option value="dialogue">Dialogue</option>
                      <option value="quest">Quest</option>
                      <option value="treasure">Treasure Hunt</option>
                      <option value="mystery">Mystery</option>
                    </select>
                  </div>

                  <div className={styles.formGroup}>
                    <label>Required Level</label>
                    <input
                      type="number"
                      value={formData.requiredLevel}
                      onChange={e => setFormData({ ...formData, requiredLevel: parseInt(e.target.value) })}
                      min="1"
                      required
                    />
                  </div>

                  <div className={styles.formGroup}>
                    <label>XP Reward</label>
                    <input
                      type="number"
                      value={formData.experienceReward}
                      onChange={e => setFormData({ ...formData, experienceReward: parseInt(e.target.value) })}
                      min="0"
                      required
                    />
                  </div>

                  <div className={styles.formGroup} style={{ gridColumn: '1 / -1' }}>
                    <label>Description</label>
                    <textarea
                      value={formData.description}
                      onChange={e => setFormData({ ...formData, description: e.target.value })}
                      rows={3}
                      required
                    />
                  </div>
                </div>

                <div className={styles.formActions}>
                  <button type="submit" className={styles.submitButton}>Save</button>
                  <button type="button" onClick={cancelEdit} className={styles.cancelButton}>Cancel</button>
                </div>
              </form>
            ) : (
              <>
                <div className={styles.adventureHeader}>
                  <h3>{adventure.name}</h3>
                  <span className={styles.badge}>{adventure.adventureType}</span>
                  <span className={styles.level}>Lvl {adventure.requiredLevel}+</span>
                </div>
                <p className={styles.description}>{adventure.description}</p>
                <div className={styles.reward}>
                  <span>XP Reward: {adventure.experienceReward}</span>
                </div>
                <div className={styles.actions}>
                  <button onClick={() => startEdit(adventure)} className={styles.editButton}>Edit</button>
                  <button onClick={() => handleDelete(adventure.id)} className={styles.deleteButton}>Delete</button>
                </div>
              </>
            )}
          </div>
        ))}
      </div>

      {adventures.length === 0 && (
        <div className={styles.empty}>
          <p>No adventures yet. Create your first adventure to get started!</p>
        </div>
      )}
    </div>
  );
}
