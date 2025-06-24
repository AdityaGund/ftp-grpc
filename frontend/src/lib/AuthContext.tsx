// src/lib/authContext.tsx
"use client";

import  { createContext, useContext, useState } from "react";
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