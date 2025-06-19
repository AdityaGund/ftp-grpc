// src/components/Unauthorized.tsx
"use client";

import React from "react";

const Unauthorized: React.FC = () => {
  return (
    <div className="flex items-center justify-center min-h-screen bg-gray-100">
      <div className="text-center p-4 bg-white shadow-md rounded-lg">
        <h1 className="text-2xl font-bold text-red-600">Unauthorized</h1>
        <p className="mt-2 text-gray-600">You do not have permission to access this page.</p>
        <a href="/login" className="mt-4 inline-block text-blue-500 hover:underline">
          Return to Login
        </a>
      </div>
    </div>
  );
};

export default Unauthorized;