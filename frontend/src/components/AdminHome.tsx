import React, { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardHeader, CardTitle, CardContent, CardDescription } from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "sonner";
import { UserPlus, Key, Globe, Info } from "lucide-react";
import axios from "axios";
import { useAuth } from "@/lib/AuthContext";
import { api } from "@/lib/api";


const AdminHome: React.FC = () => {
  useAuth();

  const [newUsername, setNewUsername] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [ip, setIp] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const [updateUsername, setUpdateUsername] = useState("");
  const [updatePassword, setUpdatePassword] = useState("");
  const [updateIp, setUpdateIp] = useState("");
  const [updating, setUpdating] = useState(false);

  const [deleteUsername, setDeleteUsername] = useState("");
  const [deleting, setDeleting] = useState(false);

  interface BankSummary { username: string; ip: string }
  interface AdminSummary { username: string }

  const [bankUsers, setBankUsers] = useState<BankSummary[]>([]);
  const [adminUsers, setAdminUsers] = useState<AdminSummary[]>([]);

  // Track currently selected user type & existing ip (for bank)
  const [selectedUserIp, setSelectedUserIp] = useState<string>("");

  const token = localStorage.getItem("jwt");

  // fetch users list
  useEffect(() => {
    const fetch = async () => {
      try {
        const res = await api.fetchUsers(token);
        const data = res.data as { banks: BankSummary[]; admins: AdminSummary[] };
        setBankUsers(data.banks);
        setAdminUsers(data.admins);
      } catch (e) {
        console.error("Failed to fetch users", e);
      }
    };
    fetch();
  }, [token, submitting, updating, deleting]);

  const handleAddUser = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newUsername || !newPassword) {
      toast.error("Username and password are required");
      return;
    }

    setSubmitting(true);
    try {
      await axios.post(
        "http://127.0.0.1:50052/api/add",
        {},
        {
          headers: {
            Authorization: `Bearer ${token}`,
            username: newUsername,
            password: newPassword,
            ip: ip || "",
          },
        }
      );
      toast.success("User added successfully");
      setNewUsername("");
      setNewPassword("");
      setIp("");
    } catch (err: unknown) {
      if (axios.isAxiosError(err)) {
        toast.error(err.response?.data?.message || "Failed to add user");
      } else {
        toast.error("Failed to add user");
      }
    } finally {
      setSubmitting(false);
    }
  };

  const handleUpdateUser = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!updateUsername) {
      toast.error("Username is required to update user");
      return;
    }
    if (!updatePassword && !updateIp) {
      toast.error("Provide at least one field to update");
      return;
    }

    setUpdating(true);
    try {
      await axios.post(
        "http://127.0.0.1:50052/api/update",
        {},
        {
          headers: {
            Authorization: `Bearer ${token}`,
            username: updateUsername,
            ...(updatePassword ? { password: updatePassword } : {}),
            ...(updateIp ? { ip: updateIp } : {}),
          },
        }
      );
      toast.success("User updated successfully");
      setUpdateUsername("");
      setUpdatePassword("");
      setUpdateIp("");
    } catch (err: unknown) {
      if (axios.isAxiosError(err)) {
        toast.error(err.response?.data?.message || "Failed to update user");
      } else {
        toast.error("Failed to update user");
      }
    } finally {
      setUpdating(false);
    }
  };

  const handleDeleteUser = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!deleteUsername) {
      toast.error("Username is required to delete user");
      return;
    }
    setDeleting(true);
    try {
      await axios.post(
        "http://127.0.0.1:50052/api/delete",
        {},
        {
          headers: {
            Authorization: `Bearer ${token}`,
            username: deleteUsername,
          },
        }
      );
      toast.success("User deleted successfully");
      setDeleteUsername("");
    } catch (err: unknown) {
      if (axios.isAxiosError(err)) {
        toast.error(err.response?.data?.message || "Failed to delete user");
      } else {
        toast.error("Failed to delete user");
      }
    } finally {
      setDeleting(false);
    }
  };

  return (
    <div className="w-full max-w-2xl mx-auto">
      <Card className="border shadow-sm">
        <CardHeader className="border-b border-border/50 pb-6">
          <CardTitle className="flex items-center gap-2 text-xl">
            <UserPlus className="h-5 w-5 text-primary" />
            <span>Add</span>
          </CardTitle>
          <CardDescription className="text-base">
            Create new Bank or Admin users for the system
          </CardDescription>
        </CardHeader>
        <CardContent className="p-6">
          <div className="mb-6 p-4 bg-blue-50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-800 rounded-lg">
            <div className="flex items-start gap-2">
              <Info className="h-4 w-4 text-blue-600 dark:text-blue-400 mt-0.5 flex-shrink-0" />
              <div className="text-sm">
                <div className="font-medium text-blue-800 dark:text-blue-200 mb-1">User Type Guidelines:</div>
                <ul className="text-blue-700 dark:text-blue-300 space-y-1">
                  <li>• <strong>Bank users:</strong> Username must start with 'B' and require an IP address</li>
                  <li>• <strong>Admin users:</strong> Username must start with 'A' and don't need an IP</li>
                </ul>
              </div>
            </div>
          </div>

          <form onSubmit={handleAddUser} className="space-y-6">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="username" className="text-sm font-medium flex items-center gap-2">
                  <UserPlus className="h-4 w-4 text-muted-foreground" />
                  Username
                </Label>
                <Input
                  id="username"
                  value={newUsername}
                  onChange={(e) => setNewUsername(e.target.value)}
                  placeholder="B(bankname) or A(adminname)"
                  disabled={submitting}
                  required
                  className="bg-background"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password" className="text-sm font-medium flex items-center gap-2">
                  <Key className="h-4 w-4 text-muted-foreground" />
                  Password
                </Label>
                <Input
                  id="password"
                  type="password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  placeholder="Enter secure password"
                  disabled={submitting}
                  required
                  className="bg-background"
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="ip" className="text-sm font-medium flex items-center gap-2">
                <Globe className="h-4 w-4 text-muted-foreground" />
                IP Address
                <span className="text-xs text-muted-foreground font-normal">(required for Bank users)</span>
              </Label>
              <Input
                id="ip"
                value={ip}
                onChange={(e) => setIp(e.target.value)}
                placeholder="192.168.1.100"
                disabled={submitting}
                className="bg-background"
              />
            </div>

            <Button 
              type="submit" 
              disabled={submitting} 
              className="w-full gap-2"
            >
              {submitting ? (
                <>
                  <Spinner size="sm" />
                  Adding User...
                </>
              ) : (
                <>
                  <UserPlus className="h-4 w-4" />
                  Add User
                </>
              )}
            </Button>
          </form>
        </CardContent>
      </Card>

      {/* UPDATE USER */}
      <Card className="border shadow-sm mt-8">
        <CardHeader className="border-b border-border/50 pb-6">
          <CardTitle className="flex items-center gap-2 text-xl">
            <UserPlus className="h-5 w-5 text-primary transform rotate-90" />
            <span>Update User</span>
          </CardTitle>
          <CardDescription className="text-base">
            Update existing Bank or Admin users
          </CardDescription>
        </CardHeader>
        <CardContent className="p-6">
          <form onSubmit={handleUpdateUser} className="space-y-6">
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <div className="space-y-2">
                <Label htmlFor="update-username">Select User</Label>
                <select
                  id="update-username"
                  value={updateUsername}
                  onChange={(e) => {
                    const username = e.target.value;
                    setUpdateUsername(username);
                    const bank = bankUsers.find((b) => b.username === username);
                    if (bank) {
                      setSelectedUserIp(bank.ip);
                      // Keep existing ip in placeholder, don't overwrite updateIp unless user already typed something
                      if (updateIp === "") {
                        setUpdateIp("");
                      }
                    } else {
                      setSelectedUserIp("");
                    }
                  }}
                  disabled={updating}
                  required
                  className="w-full border rounded-md p-2 bg-background"
                >
                  <option value="" disabled>
                    -- Select --
                  </option>
                  <optgroup label="Bank Users">
                    {bankUsers.map((u) => (
                      <option key={u.username} value={u.username}>
                        {u.username}
                      </option>
                    ))}
                  </optgroup>
                  <optgroup label="Admin Users">
                    {adminUsers.map((u) => (
                      <option key={u.username} value={u.username}>
                        {u.username}
                      </option>
                    ))}
                  </optgroup>
                </select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="update-password">New Password</Label>
                <Input
                  id="update-password"
                  value={updatePassword}
                  onChange={(e) => setUpdatePassword(e.target.value)}
                  placeholder="Leave blank to keep unchanged"
                  disabled={updating}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="update-ip">New IP (Banks only)</Label>
                <Input
                  id="update-ip"
                  value={updateIp}
                  onChange={(e) => setUpdateIp(e.target.value)}
                  placeholder={selectedUserIp ? `Current: ${selectedUserIp}` : "192.168.1.101"}
                  disabled={updating || (!updateUsername.startsWith('B'))}
                />
              </div>
            </div>
            <Button type="submit" disabled={updating} className="w-full gap-2">
              {updating ? <Spinner size="sm" /> : null}
              Update User
            </Button>
          </form>
        </CardContent>
      </Card>

      {/* DELETE USER */}
      <Card className="border shadow-sm mt-8">
        <CardHeader className="border-b border-border/50 pb-6">
          <CardTitle className="flex items-center gap-2 text-xl">
            <UserPlus className="h-5 w-5 text-destructive" />
            <span>Delete User</span>
          </CardTitle>
          <CardDescription className="text-base">Remove a user permanently</CardDescription>
        </CardHeader>
        <CardContent className="p-6">
          <form onSubmit={handleDeleteUser} className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="delete-username">Select User</Label>
              <select
                id="delete-username"
                value={deleteUsername}
                onChange={(e) => setDeleteUsername(e.target.value)}
                disabled={deleting}
                required
                className="w-full border rounded-md p-2 bg-background"
              >
                <option value="" disabled>
                  -- Select --
                </option>
                <optgroup label="Bank Users">
                  {bankUsers.map((u) => (
                    <option key={u.username} value={u.username}>
                      {u.username}
                    </option>
                  ))}
                </optgroup>
                <optgroup label="Admin Users">
                  {adminUsers.map((u) => (
                    <option key={u.username} value={u.username}>
                      {u.username}
                    </option>
                  ))}
                </optgroup>
              </select>
            </div>
            <Button type="submit" disabled={deleting} className="w-full gap-2">
              {deleting ? <Spinner size="sm" /> : null}
              Delete User
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};

export default AdminHome;