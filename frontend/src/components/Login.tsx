/* eslint-disable @typescript-eslint/no-explicit-any */
// src/components/Login.tsx
"use client";

import type React from "react";
import { useState } from "react";
import axios from "axios";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Lock, User, AlertCircle, Shield } from "lucide-react";
import { useAuth } from "../lib/AuthContext";

const Login: React.FC = () => {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const { login } = useAuth();

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    // console.log("handleLogin function called");
    setError("");
    setIsLoading(true);

    try {
      const serverURL = import.meta.env.VITE_SERVER_API_URL ?? "http://localhost:50052"
      const response = await axios.post(
        `${serverURL}/login`,
        {},
        {
          headers: {
            username,
            password,
          },
        }
      );

      const { token, role }: { token: string; role: string } = response.data;
      // console.log("Login response data:", response.data);

      if (!role || !["bank", "admin"].includes(role)) {
        throw new Error("Invalid role received from server");
      }
      // console.log("Login successful, token:", token, "role:", role);
      login(token, role as "bank" | "admin", username);
    } catch (err: any) {
      setError(err.response?.data?.message || "Login failed");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-background via-background to-muted/30 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <Card className="border shadow-2xl hover:shadow-3xl transition-all duration-300 backdrop-blur-sm">
          <CardHeader className="space-y-6 text-center pb-8">
            <div className="mx-auto w-16 h-16 bg-primary rounded-2xl flex items-center justify-center shadow-lg hover:shadow-xl transition-all duration-300 hover:scale-105">
              <Shield className="w-8 h-8 text-primary-foreground" />
            </div>
            <div className="space-y-3">
              <CardTitle className="text-3xl font-bold tracking-tight">
                File Transfer
              </CardTitle>
              <CardDescription className="text-base leading-relaxed">
                Sign in to your secure account to continue
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent className="space-y-2">
            <form onSubmit={handleLogin} className="space-y-6">
              <div className="space-y-3">
                <Label htmlFor="username" className="text-sm font-semibold cursor-pointer">
                  Username
                </Label>
                <div className="relative group">
                  <User className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors duration-200" />
                  <Input
                    type="text"
                    id="username"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    required
                    disabled={isLoading}
                    className="pl-10 h-11"
                    placeholder="Enter your username"
                  />
                </div>
              </div>

              <div className="space-y-3">
                <Label htmlFor="password" className="text-sm font-semibold cursor-pointer">
                  Password
                </Label>
                <div className="relative group">
                  <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors duration-200" />
                  <Input
                    type="password"
                    id="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                    disabled={isLoading}
                    className="pl-10 h-11"
                    placeholder="Enter your password"
                  />
                </div>
              </div>

              {error && (
                <Alert variant="destructive" className="border-destructive/20 bg-destructive/5">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription className="font-medium">{error}</AlertDescription>
                </Alert>
              )}

              <Button
                type="submit"
                disabled={isLoading}
                className="w-full h-11 text-base font-semibold"
              >
                {isLoading ? "Signing in..." : "Sign In"}
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default Login;