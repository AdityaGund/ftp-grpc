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

  // Role-based access control for known protected paths
  const path = location.pathname;
  if (path.startsWith("/adminHome") && user?.role !== "admin") {
    return <Navigate to="/unauthorized" replace />;
  }
  if (path.startsWith("/FileUpload") && user?.role !== "bank") {
    return <Navigate to="/unauthorized" replace />;
  }

  // If user navigates to root or /login while already authenticated, redirect them to their dashboard
  if (path === "/" || path === "/login") {
    if (user?.role === "admin") {
      return <Navigate to="/adminHome" replace />;
    }
    if (user?.role === "bank") {
      return <Navigate to="/FileUpload" replace />;
    }
  }

  return <Outlet />; // Allow route to render
};

export default ProtectedRoute;