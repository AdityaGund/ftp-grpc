import type React from "react"
import { ThemeToggle } from "../components/ui/ThemeToggle"
import { Button } from "@/components/ui/button"
import { useAuth } from "@/lib/AuthContext"
import { LogOut, Shield, Building2 } from "lucide-react"
import { Separator } from "@/components/ui/separator"

interface LayoutProps {
  children: React.ReactNode
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  const { logout, username, user } = useAuth()

  const getRoleIcon = () => {
    if (user?.role === "admin") {
      return <Shield className="h-4 w-4" />
    }
    return <Building2 className="h-4 w-4" />
  }

  const getRoleLabel = () => {
    if (user?.role === "admin") {
      return "Admin"
    }
    return "Bank"
  }

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 sticky top-0 z-50">
        <div className="container mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 items-center justify-between">
            <div className="flex items-center space-x-2">
              <div className="h-8 w-8 bg-primary rounded-lg flex items-center justify-center">
                <Shield className="h-4 w-4 text-primary-foreground" />
              </div>
              <h1 className="text-lg font-semibold hidden sm:block">File Transfer</h1>
              <h1 className="text-base font-semibold sm:hidden">FTP</h1>
            </div>

            <div className="flex items-center space-x-3">
              {/* User Info */}
              <div className="flex items-center space-x-2 px-3 py-2 rounded-lg bg-muted/50 border">
                <div className="flex items-center space-x-2">
                  {getRoleIcon()}
                  <div className="hidden sm:block">
                    <p className="text-sm font-medium">{username}</p>
                    <p className="text-xs text-muted-foreground">{getRoleLabel()}</p>
                  </div>
                  {/* Mobile view - compact username */}
                  <div className="sm:hidden">
                    <p className="text-xs font-medium truncate max-w-16">{username}</p>
                  </div>
                </div>
              </div>

              <Separator orientation="vertical" className="h-6" />
              
              <ThemeToggle />
              
              <Button
                variant="outline"
                size="sm"
                onClick={logout}
                className="flex items-center space-x-2 hover:bg-destructive hover:text-destructive-foreground transition-colors"
              >
                <LogOut className="h-4 w-4" />
                <span className="hidden sm:inline">Logout</span>
              </Button>
            </div>
          </div>
        </div>
      </header>
      
      <main className="container mx-auto py-6 px-4 sm:px-6 lg:px-8">
        <div className="flex justify-center">
          <div className="w-full">
            {children}
          </div>
        </div>
      </main>
    </div>
  )
}

export default Layout
