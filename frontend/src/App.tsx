import type React from "react"
import { Routes, Route, Navigate } from "react-router-dom"
import Login from "./components/Login.tsx"
import FileUpload from "./components/FileUpload.tsx"
import ProtectedRoute from "./components/ProtectRoutes.tsx"
import AdminHome from "./components/AdminHome"
import Unauthorized from "./components/Unauthorized"
import Layout from "./components/Layout.tsx"
import  {ThemeProvider}  from "./components/theme-provider"

const App: React.FC = () => {
  return (
    <ThemeProvider defaultTheme="light" storageKey="vite-ui-theme">
      <div className="min-h-screen bg-background">
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route element={<ProtectedRoute />}>
            <Route
              path="/FileUpload"
              element={
                <Layout>
                  <FileUpload />
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
      </div>
    </ThemeProvider>
  )
}

export default App
