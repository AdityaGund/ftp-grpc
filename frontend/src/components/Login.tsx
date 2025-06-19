"use client"

import type React from "react"
import { useState } from "react"
import { useNavigate } from "react-router-dom"
import axios from "axios"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Lock, User, AlertCircle, Shield } from "lucide-react"

const Login: React.FC = () => {
  const [username, setUsername] = useState("")
  const [password, setPassword] = useState("")
  const [error, setError] = useState("")
  const navigate = useNavigate()

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setError("")

    try {
      const serverHost = "127.0.0.1"
      const serverPort = "50052"
      const response = await axios.post(
        `http://${serverHost}:${serverPort}/login`,
        {},
        {
          headers: {
            username,
            password,
          },
        },
      )

      const { token } = response.data
      console.log('Login successful, token:", token')
      // Store the JWT token in local storage
      localStorage.setItem("jwt", token)

      // Redirect to /FileUpload route
      navigate("/FileUpload")
    } catch (err: any) {
      setError(err.response?.data?.message || "Login failed")
    }
  }

  return (
    <div className="w-full max-w-md mx-auto relative">
      {/* Subtle background decoration */}
      <div className="absolute -inset-4 bg-gradient-to-r from-blue-500/10 via-purple-500/10 to-emerald-500/10 rounded-3xl blur-xl opacity-60" />

      <Card className="relative bg-white/80 backdrop-blur-sm border-0 shadow-2xl shadow-slate-900/10">
        <CardHeader className="space-y-6 text-center pb-8 pt-10">
          <div className="mx-auto w-20 h-20 bg-gradient-to-br from-slate-900 via-slate-800 to-slate-700 rounded-3xl flex items-center justify-center mb-4 shadow-2xl shadow-slate-900/25 relative overflow-hidden">
            <div className="absolute inset-0 bg-gradient-to-br from-white/20 to-transparent" />
            <Shield className="w-10 h-10 text-white relative z-10" />
          </div>
          <div className="space-y-3">
            <CardTitle className="text-4xl font-bold bg-gradient-to-r from-slate-900 via-slate-800 to-slate-700 bg-clip-text text-transparent">
              Welcome Back
            </CardTitle>
            <CardDescription className="text-slate-600 text-lg font-medium">
              Sign in to your account to continue
            </CardDescription>
          </div>
        </CardHeader>

        <CardContent className="space-y-8 px-10 pb-10">
          <form onSubmit={handleLogin} className="space-y-6">
            <div className="space-y-3">
              <Label htmlFor="username" className="text-sm font-bold text-slate-800 uppercase tracking-wide">
                Username
              </Label>
              <div className="relative group">
                <div className="absolute inset-0 bg-gradient-to-r from-blue-500/20 to-purple-500/20 rounded-2xl blur opacity-0 group-focus-within:opacity-100 transition-opacity duration-300" />
                <User className="absolute left-4 top-1/2 transform -translate-y-1/2 h-5 w-5 text-slate-400 group-focus-within:text-slate-700 transition-colors duration-200 z-10" />
                <Input
                  type="text"
                  id="username"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  required
                  className="relative pl-12 h-14 border-2 border-slate-200 focus:border-slate-400 focus:ring-4 focus:ring-slate-200/50 rounded-2xl transition-all duration-300 bg-slate-50/80 focus:bg-white text-slate-800 font-medium placeholder:text-slate-400"
                  placeholder="Enter your username"
                />
              </div>
            </div>

            <div className="space-y-3">
              <Label htmlFor="password" className="text-sm font-bold text-slate-800 uppercase tracking-wide">
                Password
              </Label>
              <div className="relative group">
                <div className="absolute inset-0 bg-gradient-to-r from-blue-500/20 to-purple-500/20 rounded-2xl blur opacity-0 group-focus-within:opacity-100 transition-opacity duration-300" />
                <Lock className="absolute left-4 top-1/2 transform -translate-y-1/2 h-5 w-5 text-slate-400 group-focus-within:text-slate-700 transition-colors duration-200 z-10" />
                <Input
                  type="password"
                  id="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                  className="relative pl-12 h-14 border-2 border-slate-200 focus:border-slate-400 focus:ring-4 focus:ring-slate-200/50 rounded-2xl transition-all duration-300 bg-slate-50/80 focus:bg-white text-slate-800 font-medium placeholder:text-slate-400"
                  placeholder="Enter your password"
                />
              </div>
            </div>

            {error && (
              <Alert variant="destructive" className="border-red-300 bg-red-50/90 rounded-2xl shadow-lg">
                <AlertCircle className="h-5 w-5 text-red-600" />
                <AlertDescription className="text-red-700 font-semibold">{error}</AlertDescription>
              </Alert>
            )}

            <div className="pt-4">
              <Button
                type="submit"
                className="w-full h-14 bg-gradient-to-r from-slate-900 via-slate-800 to-slate-700 hover:from-slate-800 hover:via-slate-700 hover:to-slate-600 text-white font-bold text-lg rounded-2xl shadow-xl shadow-slate-900/25 hover:shadow-2xl hover:shadow-slate-900/30 transition-all duration-300 transform hover:scale-[1.02] active:scale-[0.98] relative overflow-hidden group"
              >
                <div className="absolute inset-0 bg-gradient-to-r from-white/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
                <span className="relative z-10">Sign In</span>
              </Button>
            </div>
          </form>

          <div className="pt-6 border-t border-slate-200/60">
            <div className="flex items-center justify-center space-x-2 text-slate-500">
              <Lock className="w-4 h-4" />
              <p className="text-sm font-medium">Secured with enterprise-grade encryption</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

export default Login
