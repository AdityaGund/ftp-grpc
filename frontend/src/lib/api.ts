import axios from 'axios';

const BASE = import.meta.env.VITE_API_URL ?? 'http://localhost:8080';

export function uploadFile(
  file: File | null,
  message: string | null,
  destination: string,
  onProgress?: (pct: number) => void
) {
  const form = new FormData();
  if (file) form.append('file', file);
  if (message) form.append('message', message);
  form.append('destination', destination);

  return axios.post(`${BASE}/upload`, form, {
    headers: { 'Content-Type': 'multipart/form-data' },
    onUploadProgress: (e) => {
      if (!onProgress) return;
      const pct = Math.round((e.loaded / (e.total ?? 1)) * 100);
      onProgress(pct);
    },
  });
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