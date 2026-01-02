import { useEffect, useState } from 'react';
import { ArrowLeft } from 'lucide-react';
import GameAPI from '../services/api';
import type { Location, Creature, Adventure } from '../types/game';
import styles from './ManageLocations.module.css';

interface ManageLocationsProps {
  onBack: () => void;
  onLogout?: () => void;
}

export default function ManageLocations({ onBack }: ManageLocationsProps) {
  const [locations, setLocations] = useState<Location[]>([]);
  const [creatures, setCreatures] = useState<Creature[]>([]);
  const [adventures, setAdventures] = useState<Adventure[]>([]);
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [formData, setFormData] = useState<Partial<Location>>({
    name: '',
    description: '',
    minLevel: 1,
    maxLevel: 10,
    tier: 1,
    enabled: true,
  });

  useEffect(() => {
    fetchData();
  }, []);

  const fetchData = async () => {
    setLoading(true);
    try {
      const [locData, creatData, advData] = await Promise.all([
        GameAPI.adminGetAllLocations(),
        GameAPI.adminGetAllCreatures(),
        GameAPI.adminGetAllAdventures(),
      ]);
      setLocations(locData);
      setCreatures(creatData);
      setAdventures(advData);
    } catch (error) {
      console.error('Failed to fetch data:', error);
      alert('Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      // Ensure all required fields are present
      const locationData = {
        name: formData.name || '',
        description: formData.description || '',
        minLevel: formData.minLevel || 1,
        maxLevel: formData.maxLevel || 10,
        tier: formData.tier || 1,
        enabled: formData.enabled ?? true,
      };
      await GameAPI.adminCreateLocation(locationData);
      setCreating(false);
      resetForm();
      fetchData();
    } catch (error) {
      console.error('Failed to create location:', error);
      alert('Failed to create location');
    }
  };

  const handleUpdate = async (id: string) => {
    try {
      // Ensure all required fields are present
      const locationData = {
        name: formData.name || '',
        description: formData.description || '',
        minLevel: formData.minLevel || 1,
        maxLevel: formData.maxLevel || 10,
        tier: formData.tier || 1,
        enabled: formData.enabled ?? true,
      };
      await GameAPI.adminUpdateLocation(id, locationData);
      setEditing(null);
      fetchData();
    } catch (error) {
      console.error('Failed to update location:', error);
      alert('Failed to update location');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this location? This will remove all associated creatures and adventures.')) return;
    
    try {
      await GameAPI.adminDeleteLocation(id);
      fetchData();
    } catch (error) {
      console.error('Failed to delete location:', error);
      alert('Failed to delete location');
    }
  };

  const startEdit = (location: Location) => {
    setEditing(location.id);
    setFormData(location);
  };

  const resetForm = () => {
    setFormData({
      name: '',
      description: '',
      minLevel: 1,
      maxLevel: 10,
      tier: 1,
      enabled: true,
    });
  };

  const cancelEdit = () => {
    setEditing(null);
    resetForm();
  };

  if (loading) {
    return <div className={styles.container}>Loading locations...</div>;
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onBack}>
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
        <h1>Location Management</h1>
        <button onClick={() => setCreating(true)} className={styles.createButton}>
          Create New Location
        </button>
      </div>

      {creating && (
        <div className={styles.modal}>
          <div className={styles.modalContent}>
            <h2>Create New Location</h2>
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
                  <label>Tier *</label>
                  <input
                    type="number"
                    value={formData.tier}
                    onChange={e => setFormData({ ...formData, tier: parseInt(e.target.value) })}
                    min="1"
                    required
                  />
                </div>

                <div className={styles.formGroup}>
                  <label>Min Level *</label>
                  <input
                    type="number"
                    value={formData.minLevel}
                    onChange={e => setFormData({ ...formData, minLevel: parseInt(e.target.value) })}
                    min="1"
                    required
                  />
                </div>

                <div className={styles.formGroup}>
                  <label>Max Level *</label>
                  <input
                    type="number"
                    value={formData.maxLevel}
                    onChange={e => setFormData({ ...formData, maxLevel: parseInt(e.target.value) })}
                    min="1"
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

                <div className={styles.formGroup} style={{ gridColumn: '1 / -1' }}>
                  <label className={styles.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={formData.enabled ?? true}
                      onChange={e => setFormData({ ...formData, enabled: e.target.checked })}
                      className={styles.checkbox}
                    />
                    <span>Enabled (visible to players)</span>
                  </label>
                </div>
              </div>

              <div className={styles.formActions}>
                <button type="submit" className={styles.submitButton}>
                  Create Location
                </button>
                <button type="button" onClick={() => { setCreating(false); resetForm(); }} className={styles.cancelButton}>
                  Cancel
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      <div className={styles.locationGrid}>
        {locations.map(location => (
          <div key={location.id} className={styles.locationCard}>
            {editing === location.id ? (
              <form onSubmit={e => { e.preventDefault(); handleUpdate(location.id); }}>
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
                    <label>Tier</label>
                    <input
                      type="number"
                      value={formData.tier}
                      onChange={e => setFormData({ ...formData, tier: parseInt(e.target.value) })}
                      min="1"
                      required
                    />
                  </div>

                  <div className={styles.formGroup}>
                    <label>Min Level</label>
                    <input
                      type="number"
                      value={formData.minLevel}
                      onChange={e => setFormData({ ...formData, minLevel: parseInt(e.target.value) })}
                      min="1"
                      required
                    />
                  </div>

                  <div className={styles.formGroup}>
                    <label>Max Level</label>
                    <input
                      type="number"
                      value={formData.maxLevel}
                      onChange={e => setFormData({ ...formData, maxLevel: parseInt(e.target.value) })}
                      min="1"
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

                  <div className={styles.formGroup} style={{ gridColumn: '1 / -1' }}>
                    <label className={styles.checkboxLabel}>
                      <input
                        type="checkbox"
                        checked={formData.enabled ?? true}
                        onChange={e => setFormData({ ...formData, enabled: e.target.checked })}
                        className={styles.checkbox}
                      />
                      <span>Enabled (visible to players)</span>
                    </label>
                  </div>
                </div>

                <div className={styles.formActions}>
                  <button type="submit" className={styles.submitButton}>Save</button>
                  <button type="button" onClick={cancelEdit} className={styles.cancelButton}>Cancel</button>
                </div>
              </form>
            ) : (
              <>
                <div className={styles.locationHeader}>
                  <h3>{location.name}</h3>
                  <div className={styles.badges}>
                    <span className={styles.tier}>Tier {location.tier}</span>
                    <span className={styles.level}>Lvl {location.minLevel}-{location.maxLevel}</span>
                  </div>
                </div>
                <p className={styles.description}>{location.description}</p>
                <div className={styles.info}>
                  <span>📍 Manage creatures and adventures (coming soon)</span>
                </div>
                <div className={styles.actions}>
                  <button onClick={() => startEdit(location)} className={styles.editButton}>Edit</button>
                  <button onClick={() => handleDelete(location.id)} className={styles.deleteButton}>Delete</button>
                </div>
              </>
            )}
          </div>
        ))}
      </div>

      {locations.length === 0 && (
        <div className={styles.empty}>
          <p>No locations yet. Create your first location to get started!</p>
        </div>
      )}
    </div>
  );
}
