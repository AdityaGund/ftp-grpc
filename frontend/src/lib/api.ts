import axios from 'axios';

// Base URL of the Bank *client* HTTP server that exposes /upload
// You can override it in your .env file using VITE_CLIENT_API_URL
const BASE = import.meta.env.VITE_CLIENT_API_URL ?? 'http://localhost:8081';

/**
 * Upload a file or message to the /upload endpoint that the Rust *client* crate exposes.
 *
 * @param file        Optional file to upload
 * @param message     Optional text message
 * @param destination Receiver bank username/id
 * @param sender      Username of the currently-authenticated bank user
 * @param token       JWT returned by the /login endpoint – required by the server for auth
 * @param onProgress  Optional callback to track upload progress (0-100)
 */
export function uploadFile(
  file: File | null,
  message: string | null,
  destination: string,
  sender: string | null,
  token: string | null,
  // onProgress?: (pct: number) => void,
) {
  const form = new FormData();
  if (file) form.append('file', file);
  if (message) form.append('message', message);
  form.append('destination', destination);
  if (sender) form.append('sender', sender);

  return axios.post(`${BASE}/upload`, form, {
    headers: {
      Authorization: `Bearer ${token}`,
    },
    // onUploadProgress: (e) => {
    //   if (!onProgress) return;
    //   const pct = Math.round((e.loaded / (e.total ?? 1)) * 100);
    //   onProgress(pct);
    // },
  });
}

export function fetchAvailableBanks(token: string | null) {
  const baseUrl = import.meta.env.VITE_SERVER_API_URL ?? 'http://127.0.0.1:50052';
  return axios.get(`${baseUrl}/api/available`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });
}

export function updateUser(
  username: string,
  newPassword: string | null,
  newIp: string | null,
  token: string | null,
) {
  const baseUrl = import.meta.env.VITE_SERVER_API_URL ?? 'http://127.0.0.1:50052';
  return axios.post(`${baseUrl}/api/update`, {}, {
    headers: {
      Authorization: `Bearer ${token}`,
      username,
      ...(newPassword ? { password: newPassword } : {}),
      ...(newIp ? { ip: newIp } : {}),
    },
  });
}

export function deleteUser(username: string, token: string | null) {
  const baseUrl = import.meta.env.VITE_SERVER_API_URL ?? 'http://127.0.0.1:50052';
  return axios.post(`${baseUrl}/api/delete`, {}, {
    headers: {
      Authorization: `Bearer ${token}`,
      username,
    },
  });
}

export function fetchUsers(token: string | null) {
  const baseUrl = import.meta.env.VITE_SERVER_API_URL ?? 'http://127.0.0.1:50052';
  return axios.get(`${baseUrl}/api/users`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });
}

/**
 * Fetch file information from the client API
 */
export function fetchFileInfo(token: string | null) {
  return axios.get(`${BASE}/file-info`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });
}

export const api = {
  uploadFile,
  fetchAvailableBanks,
  updateUser,
  deleteUser,
  fetchUsers,
  fetchFileInfo,
};