import React, { useEffect, useState, useRef } from 'react';
import { createPortal } from 'react-dom';
import { ArrowLeft, Shield, User as UserIcon, ChevronDown, Edit, Trash2 } from 'lucide-react';
import axios from 'axios';
import styles from './ManageUsers.module.css';

interface User {
  id: string;
  discordId: string;
  discordName: string;
  username?: string;
  admin: boolean;  banned: boolean;  createdAt: string;
}

interface ManageUsersProps {
  onBack: () => void;
  onLogout?: () => void;
  onAdminClick?: () => void;
}

export const ManageUsers: React.FC<ManageUsersProps> = ({ onBack, onLogout }) => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [openDropdown, setOpenDropdown] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState(false);
  const [dropdownPosition, setDropdownPosition] = useState<{ top: number; left: number } | null>(null);
  const buttonRefs = useRef<Map<string, HTMLButtonElement>>(new Map());

  useEffect(() => {
    const fetchUsers = async () => {
      try {
        setLoading(true);
        setError(null);
        const token = localStorage.getItem('authToken');
        const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';
        const response = await axios.get(`${API_BASE_URL}/admin/users`, {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        });
        // Transform backend snake_case to frontend camelCase
        const transformedUsers = response.data.map((user: any) => {
          console.log('Raw user data from backend:', user);
          return {
            id: user.id || 'unknown',
            discordId: user.discord_id || '',
            discordName: user.discord_name || '',
            username: user.username,
            dateOfBirth: user.date_of_birth,
            admin: user.admin || false,
            banned: user.banned || false,
            createdAt: user.created_at || new Date().toISOString(),
          };
        });
        console.log('Transformed users:', transformedUsers);
        setUsers(transformedUsers);
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

  // Handle dropdown toggle with positioning
  const handleDropdownToggle = (userId: string, event: React.MouseEvent<HTMLButtonElement>) => {
    if (openDropdown === userId) {
      setOpenDropdown(null);
      setDropdownPosition(null);
    } else {
      const button = event.currentTarget;
      const rect = button.getBoundingClientRect();
      setDropdownPosition({
        top: rect.bottom + window.scrollY + 8,
        left: rect.right + window.scrollX - 180, // 180 = min-width of dropdown
      });
      setOpenDropdown(userId);
      buttonRefs.current.set(userId, button);
    }
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (openDropdown) {
        const button = buttonRefs.current.get(openDropdown);
        const dropdown = document.getElementById('user-actions-dropdown');
        if (
          button &&
          dropdown &&
          !button.contains(event.target as Node) &&
          !dropdown.contains(event.target as Node)
        ) {
          setOpenDropdown(null);
          setDropdownPosition(null);
        }
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [openDropdown]);

  const handleChangeUsername = async (user: User) => {
    const newUsername = prompt(`Enter new username for ${user.discordName}:`, user.username || '');
    if (!newUsername || newUsername === user.username) return;

    try {
      setActionLoading(true);
      const token = localStorage.getItem('authToken');
      const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';
      
      await axios.patch(
        `${API_BASE_URL}/admin/users/${user.id}`,
        { username: newUsername },
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      // Update local state
      setUsers(users.map(u => u.id === user.id ? { ...u, username: newUsername } : u));
      setOpenDropdown(null);
    } catch (err) {
      console.error('Failed to update username:', err);
      alert('Failed to update username');
    } finally {
      setActionLoading(false);
    }
  };

  const handleDeleteUser = async (user: User) => {
    if (!confirm(`Are you sure you want to delete user "${user.username || user.discordName}"? This action cannot be undone.`)) {
      return;
    }

    try {
      setActionLoading(true);
      const token = localStorage.getItem('authToken');
      const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';
      
      // Check if deleting current user
      const currentUserResponse = await axios.get(`${API_BASE_URL}/user`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });
      const currentUserId = currentUserResponse.data.id;
      const isDeletingSelf = user.id === currentUserId;
      
      await axios.delete(
        `${API_BASE_URL}/admin/users/${user.id}`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      // Update local state
      setUsers(users.filter(u => u.id !== user.id));
      setOpenDropdown(null);
      
      // If user deleted themselves, log them out
      if (isDeletingSelf) {
        alert('You have deleted your own account. You will now be logged out.');
        if (onLogout) {
          onLogout();
        }
      }
    } catch (err) {
      console.error('Failed to delete user:', err);
      alert('Failed to delete user');
    } finally {
      setActionLoading(false);
    }
  };

  const handleBanUser = async (user: User) => {
    if (!confirm(`Are you sure you want to ban ${user.username || user.discordName}?`)) return;

    try {
      setActionLoading(true);
      const token = localStorage.getItem('authToken');
      const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

      const response = await axios.patch(
        `${API_BASE_URL}/admin/users/${user.id}/ban`,
        {},
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      // Update local state
      setUsers(users.map(u => u.id === user.id ? response.data : u));
      setOpenDropdown(null);
    } catch (err) {
      console.error('Failed to ban user:', err);
      alert('Failed to ban user');
    } finally {
      setActionLoading(false);
    }
  };

  const handleUnbanUser = async (user: User) => {
    try {
      setActionLoading(true);
      const token = localStorage.getItem('authToken');
      const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api';

      const response = await axios.patch(
        `${API_BASE_URL}/admin/users/${user.id}/unban`,
        {},
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      // Update local state
      setUsers(users.map(u => u.id === user.id ? response.data : u));
      setOpenDropdown(null);
    } catch (err) {
      console.error('Failed to unban user:', err);
      alert('Failed to unban user');
    } finally {
      setActionLoading(false);
    }
  };

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
                  <th>Username</th>
                  <th>Discord Name</th>
                  <th>Discord ID</th>
                  <th>Role</th>
                  <th>Created At</th>
                  <th className={styles.actionsHeader}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {users.map((user) => (
                  <tr key={user.id} className={user.admin ? styles.adminRow : ''}>
                    <td>
                      <div className={styles.nameCell}>
                        <UserIcon size={16} />
                        {user.username || <span className={styles.noUsername}>Not set</span>}
                      </div>
                    </td>
                    <td>
                      <div className={styles.nameCell}>
                        {user.discordName}
                      </div>
                    </td>
                    <td className={styles.discordId}>{user.discordId}</td>
                    <td>
                      {user.admin ? (
                        <span className={styles.adminBadge}>
                          <Shield size={14} />
                          ADMIN
                        </span>
                      ) : user.banned ? (
                        <span className={styles.bannedBadge}>BANNED</span>
                      ) : (
                        <span className={styles.userBadge}>USER</span>
                      )}
                    </td>
                    <td className={styles.dateCell}>
                      {new Date(user.createdAt).toLocaleDateString('en-US', {
                        year: 'numeric',
                        month: 'short',
                        day: 'numeric',
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </td>
                    <td className={styles.actionsCell}>
                      <div className={styles.actionsDropdown}>
                        <button
                          className={styles.actionsButton}
                          onClick={(e) => handleDropdownToggle(user.id, e)}
                          disabled={actionLoading}
                        >
                          Actions <ChevronDown size={14} />
                        </button>
                      </div>
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

      {/* Render dropdown using portal to escape parent container constraints */}
      {openDropdown && dropdownPosition && createPortal(
        <div
          id="user-actions-dropdown"
          className={styles.dropdownMenu}
          style={{
            position: 'fixed',
            top: `${dropdownPosition.top}px`,
            left: `${dropdownPosition.left}px`,
          }}
        >
          <button
            className={styles.dropdownItem}
            onClick={() => {
              const user = users.find(u => u.id === openDropdown);
              if (user) handleChangeUsername(user);
            }}
            disabled={actionLoading}
          >
            <Edit size={14} />
            Change Username
          </button>
          <button
            className={styles.dropdownItem}
            onClick={() => {
              const user = users.find(u => u.id === openDropdown);
              if (user) {
                if (user.banned) {
                  handleUnbanUser(user);
                } else {
                  handleBanUser(user);
                }
              }
            }}
            disabled={actionLoading}
          >
            <Shield size={14} />
            {users.find(u => u.id === openDropdown)?.banned ? 'Unban User' : 'Ban User'}
          </button>
          <button
            className={`${styles.dropdownItem} ${styles.deleteItem}`}
            onClick={() => {
              const user = users.find(u => u.id === openDropdown);
              if (user) handleDeleteUser(user);
            }}
            disabled={actionLoading}
          >
            <Trash2 size={14} />
            Delete User
          </button>
        </div>,
        document.body
      )}
    </div>
  );
};
