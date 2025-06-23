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
      const serverHost = "localhost";
      const serverPort = "50052";
      const response = await axios.post(
        `http://${serverHost}:${serverPort}/login`,
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
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <Card className="border shadow-lg">
          <CardHeader className="space-y-6 text-center pb-8">
            <div className="mx-auto w-16 h-16 bg-primary rounded-2xl flex items-center justify-center shadow-lg">
              <Shield className="w-8 h-8 text-primary-foreground" />
            </div>
            <div className="space-y-2">
              <CardTitle className="text-2xl font-bold">
                File Transfer
              </CardTitle>
              <CardDescription className="text-base">
                Log in to your account to continue
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleLogin} className="space-y-6">
              <div className="space-y-2">
                <Label htmlFor="username" className="text-sm font-medium">
                  Username
                </Label>
                <div className="relative">
                  <User className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    type="text"
                    id="username"
                    value={username}
                    onChange={(e) => setUsername(e.target.value)}
                    required
                    disabled={isLoading}
                    className="pl-10"
                    placeholder="Enter your username"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="password" className="text-sm font-medium">
                  Password
                </Label>
                <div className="relative">
                  <Lock className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    type="password"
                    id="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    required
                    disabled={isLoading}
                    className="pl-10"
                    placeholder="Enter your password"
                  />
                </div>
              </div>

              {error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}

              <Button
                type="submit"
                disabled={isLoading}
                className="w-full"
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