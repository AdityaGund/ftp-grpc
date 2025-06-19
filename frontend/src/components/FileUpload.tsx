import { useState, useEffect, type ChangeEvent, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { toast } from "sonner";
import axios from "axios";
import { uploadFile } from "@/lib/api";
import { Upload, User, SendHorizontal, X, RotateCcw } from "lucide-react";
import { useRef } from "react";
import { useAuth } from "@/lib/AuthContext";
interface Bank {
  username: string;
  ip: string;
}

export function FileUpload() {
  const [file, setFile] = useState<File | null>(null);
  const [message, setMessage] = useState<string>("");
  const [destination, setDestination] = useState<string>("");
  const [uploading, setUploading] = useState<boolean>(false);
  const [progress, setProgress] = useState<number>(0);
  const [banks, setBanks] = useState<Bank[]>([]);
  const [filteredBanks, setFilteredBanks] = useState<Bank[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { logout, username } = useAuth(); // Access logout and username from AuthContext

  // Fetch available banks on component mount
  useEffect(() => {
    const token = localStorage.getItem("jwt");

    axios
      .get("http://127.0.0.1:50052/api/available", {
        headers: {
          Authorization: "Bearer " + token,
        },
      })
      .then((response) => {
        console.log("Available banks:", response.data);
        setBanks(response.data); // Assuming response.data is an array of { username, ip }
        setFilteredBanks(response.data); // Initialize filtered banks
      })
      .catch((error) => {
        console.error("Error fetching available banks:", error);
        toast.error("Failed to load available banks.");
      });
  }, []);

  // Handle search/filtering of banks
  const handleSearch = (e: ChangeEvent<HTMLInputElement>) => {
    const searchTerm = e.target.value.toLowerCase();
    setDestination(e.target.value); // Update destination as user types
    if (searchTerm) {
      const filtered = banks.filter((bank) =>
        bank.username.toLowerCase().includes(searchTerm)
      );
      setFilteredBanks(filtered);
    } else {
      setFilteredBanks(banks); // Show all banks if search is empty
    }
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    const { username } = useAuth();
    console.log(username)
    if (!destination) {
      toast.error("Please enter a destination");
      return;
    }

    if (!file && !message) {
      toast.error("Please upload a file or enter a message");
      return;
    }

    setUploading(true);
    setProgress(0);

    try {
      setProgress(0);
      const response = await uploadFile(
        file,
        message || null,
        destination,
        username,
        (pct) => setProgress(pct)
      );

      toast.success("Transfer completed successfully!");
      console.log("Upload response:", response);
      setProgress(100);

      // Reset form
      setFile(null);
      setMessage("");
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    } catch (error) {
      setProgress(0);
      toast.error("Transfer failed. Please try again.");
      console.error("Upload error:", error);
    } finally {
      setUploading(false);
    }
  };

  const handleReset = () => {
    setFile(null);
    setMessage("");
    setDestination("");
    setProgress(0);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
    setFilteredBanks(banks); // Reset filtered banks to all banks
  };
  const handleLogout = () => {
    logout(); // Call the logout function
  };

  return (
    <Card className="w-full max-w-md border border-border shadow-sm">
      <CardHeader className="border-b border-border/50 pb-4">
        <CardTitle className="flex items-center gap-2">
          <Upload className="h-5 w-5 text-primary" />
          <span>File Transfer</span>
        </CardTitle>
        <CardDescription>
          Transfer files or messages securely via gRPC
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-5">
          <div className="space-y-2">
            <label htmlFor="destination" className="text-sm font-medium flex items-center gap-2">
              <User className="h-4 w-4 text-muted-foreground" />
              <span>Destination</span>
            </label>
            <Input
              id="destination"
              placeholder="Search or enter destination ID"
              value={destination}
              onChange={handleSearch}
              disabled={uploading}
              required
              className="bg-background"
            />
            {destination && filteredBanks.length > 0 && (
              <ul className="mt-1 border border-border/50 rounded-md bg-background max-h-40 overflow-y-auto">
                {filteredBanks.map((bank, index) => (
                  <li
                    key={index}
                    className="px-3 py-2 text-sm hover:bg-muted cursor-pointer"
                    onClick={() => {
                      setDestination(bank.username);
                      setFilteredBanks([]); // Clear suggestions after selection
                    }}
                  >
                    {bank.username} ({bank.ip})
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="space-y-2">
            <label htmlFor="file" className="text-sm font-medium flex items-center gap-2">
              <Upload className="h-4 w-4 text-muted-foreground" />
              <span>File</span>
            </label>
            <div className="relative">
              <Input
                ref={fileInputRef}
                id="file"
                type="file"
                onChange={(e) => {
                  if (e.target.files && e.target.files[0]) {
                    setFile(e.target.files[0]);
                  }
                }}
                disabled={uploading}
                className="bg-background"
              />
              {file && (
                <Button
                  size="icon"
                  variant="ghost"
                  className="absolute right-2 top-1/2 -translate-y-1/2 h-6 w-6"
                  onClick={() => {
                    setFile(null);
                    if (fileInputRef.current) fileInputRef.current.value = "";
                  }}
                  type="button"
                  disabled={uploading}
                >
                  <X className="h-3 w-3" />
                </Button>
              )}
            </div>
            {file && (
              <p className="text-xs text-muted-foreground">
                Selected: {file.name} ({(file.size / 1024).toFixed(2)} KB)
              </p>
            )}
          </div>

          <div className="space-y-2">
            <label htmlFor="message" className="text-sm font-medium flex items-center gap-2">
              <SendHorizontal className="h-4 w-4 text-muted-foreground" />
              <span>Message</span>
            </label>
            <Textarea
              id="message"
              placeholder="Enter a message (optional)"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              disabled={uploading}
              rows={3}
              className="bg-background resize-none"
            />
          </div>

          {uploading && (
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <label className="text-sm font-medium">Upload Progress</label>
                <span className="text-xs text-muted-foreground">{progress}%</span>
              </div>
              <Progress value={progress} className="h-2" />
            </div>
          )}

          <div className="flex space-x-3 pt-2">
            <Button
              type="submit"
              disabled={uploading}
              className="flex-1 gap-2"
            >
              {uploading ? "Uploading..." : "Send"}
              <SendHorizontal className="h-4 w-4" />
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={handleReset}
              disabled={uploading}
              className="gap-2"
            >
              Reset
              <RotateCcw className="h-4 w-4" />
            </Button>
            <Button
              type="button"
              variant="destructive"
              onClick={handleLogout}
              className="gap-2"
            >
              Logout
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}

export default FileUpload;