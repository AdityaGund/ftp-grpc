import React, { useState, useEffect } from "react";
import { toast } from "sonner";
import { useAuth } from "@/lib/AuthContext";
import { api } from "@/lib/api";
import FileInfoDisplay from "./ui/FileInfo";
import AdminUserManagement from "./ui/AdminUserManagement";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Button } from "./ui/button";
import { FileText, User } from "lucide-react";


const AdminHome: React.FC = () => {

  useAuth();

  const [activeTab, setActiveTab] = useState<"userManagement" | "files">("userManagement");


  interface FileInfo {
    _id: string | object;
    name: string;
    path: string;
    sender_bank_id: string;
    receiver_bank_id: string;
    message: string;
    time_sent_at: string;
    time_received_at: string;
  }

  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loadingFiles, setLoadingFiles] = useState<boolean>(false);

  const token = localStorage.getItem("jwt");

  // Fetch file information from admin server
  const loadFileInfo = async () => {
    setLoadingFiles(true);
    try {
      const response = await api.fetchAdminFileInfo(token);
      setFiles(response.data.data || []);
    } catch (error) {
      console.error("Error fetching admin file info", error);
      toast.error("Failed to load file history");
    } finally {
      setLoadingFiles(false);
    }
  };

  // Fetch once on mount
  useEffect(() => {
    loadFileInfo();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleRefreshFiles = () => {
    toast.success("Refreshing file information...");
    loadFileInfo();
  };

  return (
    <div className="w-full max-w-2xl mx-auto">

      <Card className="border-2 border-border shadow-md mb-6">
        <CardHeader className="border-b-2 border-border pb-4">
          <CardTitle className="text-xl">Bank Dashboard</CardTitle>
        </CardHeader>
        <CardContent className="p-6">
          <div className="flex space-x-2 mb-6">
            <Button
              variant={activeTab === "userManagement" ? "default" : "outline"}
              onClick={() => setActiveTab("userManagement")}
              className="flex items-center gap-2"
            >
              <User className="h-4 w-4" />
              User Management
            </Button>
            <Button
              variant={activeTab === "files" ? "default" : "outline"}
              onClick={() => setActiveTab("files")}
              className="flex items-center gap-2"
            >
              <FileText className="h-4 w-4" />
              File History
            </Button>
          </div>
        </CardContent>
      </Card>


      {activeTab === "userManagement" && <AdminUserManagement token={token} />}
      {activeTab === "files" &&
        <div className="mt-8">
          <FileInfoDisplay
            files={files}
            loading={loadingFiles}
            onRefresh={handleRefreshFiles}
            title="Global File Transfer History"
            description="View all file transfers across all banks."
          />
        </div>
      }
    </div>
  );
};

export default AdminHome;