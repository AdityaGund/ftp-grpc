// src/lib/authContext.tsx
"use client";

import  { createContext, useContext, useState, useEffect } from "react";
import type { ReactNode } from "react";
import { useNavigate } from "react-router-dom";

interface User {
  role: "bank" | "admin";
}

interface AuthContextType {
  isAuthenticated: boolean;
  user: User | null;
  username: string | null;
  login: (token: string, role: "bank" | "admin", username: string) => void;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

// Helper to extract the expiration time from a JWT (returns ms since epoch)
const getTokenExpiration = (token: string | null): number | null => {
  if (!token) return null;
  try {
    const payloadBase64 = token.split('.')[1];
    if (!payloadBase64) return null;
    // Convert from base64url to base64 and decode
    const payloadJson = atob(payloadBase64.replace(/-/g, '+').replace(/_/g, '/'));
    const payload = JSON.parse(payloadJson);
    if (typeof payload.exp === 'number') {
      return payload.exp * 1000; // exp is in seconds -> convert to ms
    }
  } catch {
    // Ignore malformed tokens
  }
  return null;
};

export const AuthProvider = ({ children }: { children: ReactNode }) => {
  // Initialise auth state from localStorage so that refreshes keep the user logged-in
  const [isAuthenticated, setIsAuthenticated] = useState(() => !!localStorage.getItem("jwt"));

  const [user, setUser] = useState<User | null>(() => {
    const storedRole = localStorage.getItem("role");
    return storedRole === "bank" || storedRole === "admin" ? { role: storedRole } as User : null;
  });

  const [username, setUsername] = useState<string | null>(() => localStorage.getItem("username"));

  const navigate = useNavigate();

  const login = (token: string, role: "bank" | "admin", username: string) => {
    localStorage.setItem("jwt", token);
    localStorage.setItem("role", role);
    localStorage.setItem("username", username);

    setIsAuthenticated(true);
    setUser({ role });
    setUsername(username);

    if (role === "admin") {
      navigate("/adminHome");
    } else if (role === "bank") {
      navigate("/bank");
    }
  };

  const logout = () => {
    localStorage.removeItem("jwt");
    localStorage.removeItem("role");
    localStorage.removeItem("username");
    setIsAuthenticated(false);
    setUser(null);
    navigate("/login"); // Redirect to login on logout
  };

  // Automatically log the user out when the JWT expires
  // eslint-disable-next-line react-hooks/exhaustive-deps
  useEffect(() => {
    const token = localStorage.getItem("jwt");
    const expMs = getTokenExpiration(token);

    // If we can't determine expiry, do nothing
    if (!token || !expMs) return;

    // If token already expired -> immediate logout
    if (Date.now() >= expMs) {
      logout();
      return;
    }

    // Otherwise, schedule a logout when the token expires
    const timeoutId = window.setTimeout(() => {
      logout();
    }, expMs - Date.now());

    // Clear timeout if component unmounts or dependencies change
    return () => clearTimeout(timeoutId);
  }, [isAuthenticated]);

  return (
    <AuthContext.Provider value={{ isAuthenticated, user, login, logout ,username }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) throw new Error("useAuth must be used within an AuthProvider");
  return context;
};