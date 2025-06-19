import axios from 'axios';
import { useAuth } from './AuthContext';

const BASE = import.meta.env.VITE_API_URL ?? 'http://localhost:8080';



export function uploadFile(
  file: File | null,
  message: string | null,
  destination: string,
  sender: string | null,
  onProgress?: (pct: number) => void
) {
  const form = new FormData();
  if (file) form.append('file', file);
  if (message) form.append('message', message);
  form.append('destination', destination);
  const { username } = useAuth();
  if (username) {
    form.append('sender', username);
  }

  console.log(form);

  return axios.post(`${BASE}/upload`, form, {
    headers: { 'Content-Type': 'multipart/form-data' },
    onUploadProgress: (e) => {
      if (!onProgress) return;
      const pct = Math.round((e.loaded / (e.total ?? 1)) * 100);
      onProgress(pct);
    },
  });
}

export function get_banks() {
  let res = axios.get(`http:127.0.0.1:50052/api/available`);
  console.log(res);
}


// export function openEventStream(onMsg: (ev: unknown) => void) {
//   const es = new EventSource(`${BASE}/events`);
//   es.onmessage = (e) => {
//     try {

//       // console.log(e.data);
//       const payload = JSON.parse(e.data);
//       // console.log('[ACK]', payload);
//       onMsg(payload);
//     } catch (err) {
//       console.error('Failed to parse event data', err);
//     }
//   };
//   return () => es.close();
// }

export const api={
  uploadFile,
  // openEventStream,
}