import React, { useEffect, useState } from 'react';
import { ArrowLeft, Shield, User } from 'lucide-react';
import axios from 'axios';
import styles from './ManageUsers.module.css';

interface User {
  id: string;
  discord_id: string;
  discord_name: string;
  admin: boolean;
  created_at: string;
}

interface ManageUsersProps {
  onBack: () => void;
}

export const ManageUsers: React.FC<ManageUsersProps> = ({ onBack }) => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchUsers = async () => {
      try {
        setLoading(true);
        setError(null);
        const token = localStorage.getItem('authToken');
        const response = await axios.get('http://localhost:8080/api/admin/users', {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        });
        setUsers(response.data);
      } catch (err) {
        console.error('Failed to fetch users:', err);
        if (axios.isAxiosError(err) && err.response?.status === 403) {
          setError('Access denied - Admin privileges required');
        } else {
          setError('Failed to load users');
        }
      } finally {
        setLoading(false);
      }
    };

    fetchUsers();
  }, []);

  return (
    <div className={styles.manageUsersContainer}>
      <div className={styles.header}>
        <button className={styles.backButton} onClick={onBack}>
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>
        <h1 className={styles.title}>👥 Manage Users</h1>
      </div>

      <div className={styles.content}>
        {loading && (
          <div className={styles.loading}>
            <div className={styles.loadingText}>Loading users...</div>
          </div>
        )}

        {error && (
          <div className={styles.error}>
            <span className={styles.errorIcon}>⚠️</span>
            <span>{error}</span>
          </div>
        )}

        {!loading && !error && (
          <div className={styles.tableContainer}>
            <div className={styles.tableHeader}>
              <div className={styles.statsBar}>
                Total Users: <span className={styles.highlight}>{users.length}</span>
                {' | '}
                Admins: <span className={styles.highlight}>{users.filter(u => u.admin).length}</span>
              </div>
            </div>

            <table className={styles.userTable}>
              <thead>
                <tr>
                  <th>Discord Name</th>
                  <th>Discord ID</th>
                  <th>Role</th>
                  <th>Created At</th>
                </tr>
              </thead>
              <tbody>
                {users.map((user) => (
                  <tr key={user.id} className={user.admin ? styles.adminRow : ''}>
                    <td>
                      <div className={styles.nameCell}>
                        <User size={16} />
                        {user.discord_name}
                      </div>
                    </td>
                    <td className={styles.discordId}>{user.discord_id}</td>
                    <td>
                      {user.admin ? (
                        <span className={styles.adminBadge}>
                          <Shield size={14} />
                          ADMIN
                        </span>
                      ) : (
                        <span className={styles.userBadge}>USER</span>
                      )}
                    </td>
                    <td className={styles.dateCell}>
                      {new Date(user.created_at).toLocaleDateString('en-US', {
                        year: 'numeric',
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>

            {users.length === 0 && (
              <div className={styles.emptyState}>
                No users found in the database.
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};
