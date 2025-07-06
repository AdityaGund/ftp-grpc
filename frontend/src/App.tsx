import type React from "react"
import { Routes, Route, Navigate } from "react-router-dom"
import Login from "./components/Login.tsx"
import BankHome from "./components/BankHome.tsx"
import ProtectedRoute from "./components/ProtectRoutes.tsx"
import AdminHome from "./components/AdminHome"
import Unauthorized from "./components/Unauthorized"
import Layout from "./components/Layout.tsx"
import  {ThemeProvider}  from "./components/ui/theme-provider.tsx"
import { Toaster } from "@/components/ui/sonner"
import './index.css';

const App: React.FC = () => {
  return (
    <ThemeProvider defaultTheme="system" storageKey="vite-ui-theme">
      {/* Global toast container */}
      <Toaster richColors closeButton />
      <Routes>
        <Route path="/login" element={<Login />} />
        
        <Route element={<ProtectedRoute />}>
          <Route
            path="/bank"
            element={
              <Layout>
                <BankHome />
              </Layout>
            }
          />
          <Route
            path="/adminHome"
            element={
              <Layout>
                <AdminHome />
              </Layout>
            }
          />
        </Route>
        
        <Route path="/unauthorized" element={<Unauthorized />} />
        <Route path="/" element={<Navigate to="/login" replace />} />
      </Routes>
    </ThemeProvider>
  )
}

export default App
