// src/pages/AdminHome.tsx or src/components/AdminHome.tsx

import React from "react";
import { useAuth } from "@/lib/AuthContext"; // Adjust the import path based on your structure
import { Button } from "@/components/ui/button"; // Assuming you use the same UI library

const AdminHome: React.FC = () => {
    const { logout } = useAuth(); // Access the logout function from AuthContext

    const handleLogout = () => {
    logout(); // Call the logout function
    };
    return (
        <div style={{ padding: "2rem", textAlign: "center" }}>
          <h1>Welcome to Admin Dashboard</h1>
          <p>You have admin privileges.</p>
          <Button
            onClick={handleLogout}
            className="mt-4 bg-red-600 text-white hover:bg-red-700"
          >
            Logout
          </Button>
        </div>
      );
    };
    
    export default AdminHome;