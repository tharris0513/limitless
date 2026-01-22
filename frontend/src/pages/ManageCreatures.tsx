import { useEffect, useState } from 'react';
import { ArrowLeft } from 'lucide-react';
import GameAPI from '../services/api';
import type { Creature } from '../types/game';
import styles from './ManageCreatures.module.css';

interface ManageCreaturesProps {
  onBack: () => void;
  onLogout?: () => void;
}

export default function ManageCreatures({ onBack }: ManageCreaturesProps) {
  const [creatures, setCreatures] = useState<Creature[]>([]);
  const [loading, setLoading] = useState(true);
  const [editing, setEditing] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [formData, setFormData] = useState<Partial<Creature>>({
    name: '',
    introductionText: '',
    level: 1,
    maxHealth: 100,
    might: 10,
    defense: 5,
    magic: 5,
    resistance: 5,
    agility: 10,
    experienceReward: 50,
    creatureType: 'beast',
    attackDescription: '',
  });

  useEffect(() => {
    fetchCreatures();
  }, []);

  const fetchCreatures = async () => {
    setLoading(true);
    try {
      const data = await GameAPI.adminGetAllCreatures();
      setCreatures(data);
    } catch (error) {
      console.error('Failed to fetch creatures:', error);
      alert('Failed to load creatures');
    } finally {
      setLoading(false);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    try {
      await GameAPI.adminCreateCreature(formData);
      setCreating(false);
      setFormData({
        name: '',
        introductionText: '',
        level: 1,
        maxHealth: 100,
        might: 10,
        defense: 5,
        magic: 5,
        resistance: 5,
        agility: 10,
        experienceReward: 50,
        creatureType: 'beast',
        attackDescription: '',
      });
      fetchCreatures();
    } catch (error) {
      console.error('Failed to create creature:', error);
      alert('Failed to create creature');
    }
  };

  const handleUpdate = async (id: string) => {
    try {
      await GameAPI.adminUpdateCreature(id, formData);
      setEditing(null);
      fetchCreatures();
    } catch (error) {
      console.error('Failed to update creature:', error);
      alert('Failed to update creature');
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this creature?')) return;
    
    try {
      await GameAPI.adminDeleteCreature(id);
      fetchCreatures();
    } catch (error) {
      console.error('Failed to delete creature:', error);
      alert('Failed to delete creature');
    }
  };

  const startEdit = (creature: Creature) => {
    setEditing(creature.id);
    setFormData(creature);
  };

  const cancelEdit = () => {
    setEditing(null);
    setFormData({
      name: '',
      introductionText: '',
      level: 1,
      maxHealth: 100,
      might: 10,
      defense: 5,
      magic: 5,
      resistance: 5,
      agility: 10,
      experienceReward: 50,
      creatureType: 'beast',
      attackDescription: '',
    });
  };

  if (loading) {
    return <div className={styles.container}>Loading creatures...</div>;
  }

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onBack}>
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
        <h1>Creature Management</h1>
        <button onClick={() => setCreating(true)} className={styles.createButton}>
          Create New Creature
        </button>
      </div>

      {(creating || editing) && (
        <div className={styles.modal}>
          <div className={styles.modalContent}>
            <h2>{creating ? 'Create New Creature' : 'Edit Creature'}</h2>
            <form onSubmit={creating ? handleCreate : (e) => { e.preventDefault(); handleUpdate(editing!); }}>
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
                    value={formData.creatureType}
                    onChange={e => setFormData({ ...formData, creatureType: e.target.value })}
                    required
                  >
                    <option value="beast">Beast</option>
                    <option value="demon">Demon</option>
                    <option value="dragon">Dragon</option>
                    <option value="elemental">Elemental</option>
                    <option value="horror">Horror</option>
                    <option value="humanoid">Humanoid</option>
                    <option value="undead">Undead</option>
                  </select>
                </div>

                <div className={styles.formGroup}>
                  <label>Level *</label>
                  <input
                    type="number"
                    value={formData.level}
                    onChange={e => setFormData({ ...formData, level: parseInt(e.target.value) })}
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
                  <label>Introduction Text *</label>
                  <textarea
                    value={formData.introductionText}
                    onChange={e => setFormData({ ...formData, introductionText: e.target.value })}
                    rows={3}
                    required
                  />
                </div>

                <div className={styles.formGroup} style={{ gridColumn: '1 / -1' }}>
                  <label>Attack Description (Optional)</label>
                  <input
                    type="text"
                    value={formData.attackDescription || ''}
                    onChange={e => setFormData({ ...formData, attackDescription: e.target.value })}
                    placeholder="e.g., The ${name} lunges at you for ${damage} damage!"
                  />
                  <small style={{ color: '#888', fontSize: '0.85em', marginTop: '4px', display: 'block' }}>
                    Use {`\${damage}`} or {`\${x}`} for damage, {`\${name}`} for creature name
                  </small>
                </div>

                <div className={styles.formGroup}>
                  <label>Max Health *</label>
                  <input
                    type="number"
                    value={formData.maxHealth}
                    onChange={e => setFormData({ ...formData, maxHealth: parseInt(e.target.value) })}
                    min="1"
                    required
                  />
                </div>

                <div className={styles.formGroup}>
                  <label>Might *</label>
                  <input
                    type="number"
                    value={formData.might}
                    onChange={e => setFormData({ ...formData, might: parseInt(e.target.value) })}
                    min="0"
                    required
                  />
                </div>

                <div className={styles.formGroup}>
                  <label>Defense *</label>
                  <input
                    type="number"
                    value={formData.defense}
                    onChange={e => setFormData({ ...formData, defense: parseInt(e.target.value) })}
                    min="0"
                    required
                  />
                </div>

                <div className={styles.formGroup}>
                  <label>Magic *</label>
                  <input
                    type="number"
                    value={formData.magic}
                    onChange={e => setFormData({ ...formData, magic: parseInt(e.target.value) })}
                    min="0"
                    required
                  />
                </div>

                <div className={styles.formGroup}>
                  <label>Resistance *</label>
                  <input
                    type="number"
                    value={formData.resistance}
                    onChange={e => setFormData({ ...formData, resistance: parseInt(e.target.value) })}
                    min="0"
                    required
                  />
                </div>

                <div className={styles.formGroup}>
                  <label>Agility *</label>
                  <input
                    type="number"
                    value={formData.agility}
                    onChange={e => setFormData({ ...formData, agility: parseInt(e.target.value) })}
                    min="0"
                    required
                  />
                </div>
              </div>

              <div className={styles.formActions}>
                <button type="submit" className={styles.submitButton}>
                  {creating ? 'Create Creature' : 'Save Changes'}
                </button>
                <button type="button" onClick={creating ? () => setCreating(false) : cancelEdit} className={styles.cancelButton}>
                  Cancel
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      <div className={styles.creatureGrid}>
        {creatures.map(creature => (
          <div key={creature.id} className={styles.creatureCard}>
            <div className={styles.creatureHeader}>
              <h3>{creature.name}</h3>
              <span className={styles.badge}>{creature.creatureType}</span>
              <span className={styles.level}>Lvl {creature.level}</span>
            </div>
            <div className={styles.stats}>
              <div className={styles.statRow}>
                <span>HP: {creature.maxHealth}</span>
                <span>Might: {creature.might}</span>
              </div>
              <div className={styles.statRow}>
                <span>Defense: {creature.defense}</span>
                <span>Magic: {creature.magic}</span>
              </div>
              <div className={styles.statRow}>
                <span>Resistance: {creature.resistance}</span>
                <span>Agility: {creature.agility}</span>
              </div>
              <div className={styles.statRow}>
                <span>XP: {creature.experienceReward}</span>
              </div>
            </div>
            <div className={styles.actions}>
              <button onClick={() => startEdit(creature)} className={styles.editButton}>Edit</button>
              <button onClick={() => handleDelete(creature.id)} className={styles.deleteButton}>Delete</button>
            </div>
          </div>
        ))}
      </div>

      {creatures.length === 0 && (
        <div className={styles.empty}>
          <p>No creatures yet. Create your first creature to get started!</p>
        </div>
      )}
    </div>
  );
}
