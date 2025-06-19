// src/components/ProtectedRoute.tsx
"use client";

import React from "react";
import { Navigate, Outlet, useLocation } from "react-router-dom";
import { useAuth } from "../lib/AuthContext"; // Fixed import case sensitivity

interface ProtectedRouteProps {
}

const ProtectedRoute: React.FC<ProtectedRouteProps> = () => {
  const { isAuthenticated, user } = useAuth();
  const location = useLocation();

  if (!isAuthenticated) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  // Redirect based on role if no specific route is matched (e.g., root or unmatched)
  if (!location.pathname || location.pathname === "/login") {
    if (user?.role === "bank") {
      return <Navigate to="/FileUpload" replace />;
    } else if (user?.role === "admin") {
      return <Navigate to="/adminHome" replace />;
    }
  }

  // Allow the matched route to render if authenticated
  return <Outlet />;
};

export default ProtectedRoute;