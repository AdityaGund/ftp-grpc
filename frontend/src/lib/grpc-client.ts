/* eslint-disable @typescript-eslint/no-unused-vars */
// gRPC client service for file transfer
// import * as grpcWeb from 'grpc-web';

// Define interfaces based on the proto file
// interface FileInfo {
//   name: string;
//   path: string;
//   size: number;
//   content_type: string;
// }

// interface MessageInfo {
//   length: number;
// }

// interface Metadata {
//   transfer_id: string;
//   sender_bank_id: string;
//   receiver_bank_id: string;
//   chunk_index: number;
//   total_chunks: number;
//   timestamp: string;
//   file_info?: FileInfo;
//   message_info?: MessageInfo;
// }

// interface TransferRequest {
//   metadata: Metadata;
//   content: Uint8Array;
// }

// interface ErrorInfo {
//   error_code: string;
//   error_details: string;
// }

// interface TransferResponse {
//   transfer_id: string;
//   status: 'SUCCESS' | 'IN_PROGRESS' | 'FAILURE' | 'RETRY';
//   error_info?: ErrorInfo;
// }

// This is a simplified version as we'll be using the REST API
export const uploadFile = async (
  file: File | null, 
  message: string | null, 
  destination: string
): Promise<unknown> => {
  const formData = new FormData();
  
  if (file) {
    formData.append('file', file);
  }
  
  if (message) {
    formData.append('message', message);
  }
  
  formData.append('destination', destination);

  // Get the backend URL from environment or use a default
  const backendUrl = import.meta.env.VITE_BACKEND_URL || 'http://localhost:8080';
  
  try {
    const response = await fetch(`${backendUrl}/upload`, {
      method: 'POST',
      body: formData,
    });

    console.log(response);

    if (!response.ok) {
      throw new Error(`HTTP error! Status: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error('Error uploading file:', error);
    throw error;
  }
}; 