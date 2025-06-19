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
  username: string | null; // Added username field
  login: (token: string, role: "bank" | "admin", username: string) => void; // Updated login function signature
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider = ({ children }: { children: ReactNode }) => {
  const [isAuthenticated, setIsAuthenticated] = useState(!!localStorage.getItem("jwt"));
  const [user, setUser] = useState<User | null>(null);
  const [username, setUsername] = useState<string | null>(null); // Added username state
  const navigate = useNavigate();

  // src/lib/authContext.tsx
const login = (token: string, role: "bank" | "admin", username: string) => {
  localStorage.setItem("jwt", token);
  setIsAuthenticated(true);
  setUser({ role });
  setUsername(username);
  // Redirect based on role
  if (role === "admin") {
    navigate("/adminHome");
  } else if (role === "bank") {
    navigate("/FileUpload");
  }
};

  const logout = () => {
    localStorage.removeItem("jwt");
    setIsAuthenticated(false);
    setUser(null);
    navigate("/login"); // Redirect to login on logout
  };

  return (
    <AuthContext.Provider value={{ isAuthenticated, user, login, logout ,username}}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) throw new Error("useAuth must be used within an AuthProvider");
  return context;
};