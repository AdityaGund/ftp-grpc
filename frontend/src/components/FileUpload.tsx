/* eslint-disable @typescript-eslint/no-unused-vars */
import { useState, useEffect, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { toast } from "sonner";
import { uploadFile, fetchAvailableBanks } from "@/lib/api";
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
  const [selectedBank, setSelectedBank] = useState<Bank | null>(null);
  const [uploading, setUploading] = useState<boolean>(false);
  // const [progress, setProgress] = useState<number>(0);
  const [banks, setBanks] = useState<Bank[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { username, user } = useAuth();
  const token = localStorage.getItem("jwt");

  useEffect(() => {
    fetchAvailableBanks(token)
      .then((response) => {
        console.log("Available banks:", response.data);
        setBanks(response.data);
      })
      .catch((error) => {
        console.error("Error fetching available banks:", error);
        toast.error("Failed to load available banks.");
      });
  }, []);

  const handleDestinationChange = (bank: Bank | null) => {
    setSelectedBank(bank);
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (!selectedBank) {
      toast.error("Please select a destination bank");
      return;
    }

    if (!file && !message) {
      toast.error("Please upload a file or enter a message");
      return;
    }

    setUploading(true);

    try {
      const response = await uploadFile(
        file,
        message || null,
        selectedBank.username,
        selectedBank.ip,
        username,
        token,
        user?.role === 'admin' ? 'admin' : 'bank',
        // (pct) => setProgress(pct)
      );

      toast.success("Transfer completed successfully!");
      console.log("Upload response:", response);

      // Reset form
      setFile(null);
      setMessage("");
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    } catch (error) {
      // setProgress(0);
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
    setSelectedBank(null)
    // setProgress(0);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  };

  const handleFileSelect = () => {
    if (fileInputRef.current) {
      fileInputRef.current.click();
    }
  };

  return (
    <div className="w-full max-w-3xl mx-auto">
      <Card className="border shadow-lg hover:shadow-xl transition-all duration-300">
        <CardContent className="p-8">
          <form onSubmit={handleSubmit} className="space-y-8">
            <div className="space-y-4">
              <label htmlFor="destination" className="text-sm font-semibold flex items-center gap-2 cursor-pointer">
                <User className="h-4 w-4 text-muted-foreground" />
                <span>Destination Bank</span>
              </label>
              <div className="relative">
                <select
                  id="destination"
                  value={selectedBank?.username || ""}
                  onChange={(e) => {
                    const bank = banks.find(b => b.username === e.target.value) || null;
                    handleDestinationChange(bank);
                  }}
                  disabled={uploading}
                  required
                  className="w-full p-3 border rounded-lg bg-background hover:border-ring/60 focus:border-ring focus:ring-ring/50 focus:ring-[3px] transition-all duration-200 cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
                >
                  <option value="" disabled>-- Select a destination bank --</option>
                  {banks.map((bank) => (
                    <option key={bank.username} value={bank.username}>
                      {bank.username} ({bank.ip})
                    </option>
                  ))}
                </select>
              </div>
              {selectedBank && (
                <div className="text-sm text-muted-foreground bg-muted/30 p-3 rounded-lg border">
                  <strong>Selected:</strong> {selectedBank.username} • <strong>IP:</strong> {selectedBank.ip}
                </div>
              )}
            </div>

            <div className="space-y-4">
              <label htmlFor="file" className="text-sm font-semibold flex items-center gap-2 cursor-pointer">
                <Upload className="h-4 w-4 text-muted-foreground" />
                <span>File Upload</span>
              </label>
              <div className="space-y-3">
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
                  className="hidden"
                />
                <div 
                  onClick={handleFileSelect}
                  className="border-2 border-dashed border-border hover:border-primary/50 transition-all duration-200 rounded-lg p-6 text-center cursor-pointer hover:bg-muted/30 group"
                >
                  <Upload className="mx-auto h-8 w-8 text-muted-foreground group-hover:text-primary transition-colors duration-200" />
                  <p className="mt-2 text-sm text-muted-foreground group-hover:text-foreground transition-colors duration-200">
                    Click to select a file or drag and drop
                  </p>
                  <p className="text-xs text-muted-foreground mt-1">
                    Supports various file formats
                  </p>
                </div>
                {file && (
                  <div className="flex items-center justify-between p-4 bg-muted/50 rounded-lg border">
                    <div className="flex items-center gap-3">
                      <div className="p-2 bg-primary/10 rounded-md">
                        <Upload className="h-4 w-4 text-primary" />
                      </div>
                      <div>
                        <p className="text-sm font-medium">{file.name}</p>
                        <p className="text-xs text-muted-foreground">
                          {(file.size / 1024).toFixed(2)} KB
                        </p>
                      </div>
                    </div>
                    <Button
                      size="icon"
                      variant="ghost"
                      className="h-8 w-8 hover:bg-destructive/10 hover:text-destructive"
                      onClick={() => {
                        setFile(null);
                        if (fileInputRef.current) fileInputRef.current.value = "";
                      }}
                      type="button"
                      disabled={uploading}
                    >
                      <X className="h-4 w-4" />
                    </Button>
                  </div>
                )}
              </div>
            </div>

            <div className="space-y-4">
              <label htmlFor="message" className="text-sm font-semibold flex items-center gap-2 cursor-pointer">
                <SendHorizontal className="h-4 w-4 text-muted-foreground" />
                <span>Message (Optional)</span>
              </label>
              <Textarea
                id="message"
                placeholder="Enter an optional message to accompany your transfer..."
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                disabled={uploading}
                rows={4}
                className="bg-background resize-none hover:border-ring/60 focus:border-ring transition-all duration-200"
              />
            </div>

            <div className="flex flex-col sm:flex-row gap-4 pt-6">
              <Button
                type="submit"
                disabled={uploading || (!file && !message)}
                className="flex-1 gap-2 h-11 text-base font-semibold"
              >
                {uploading ? (
                  <>
                    <Spinner size="sm" />
                    Transferring...
                  </>
                ) : (
                  <>
                    <SendHorizontal className="h-5 w-5" />
                    Send Transfer
                  </>
                )}
              </Button>
              <Button
                type="button"
                variant="outline"
                onClick={handleReset}
                disabled={uploading}
                className="gap-2 h-11 min-w-[120px] font-medium"
              >
                <RotateCcw className="h-4 w-4" />
                Reset Form
              </Button>
            </div>

            {(!file && !message) && (
              <div className="text-center p-4 bg-muted/20 rounded-lg border">
                <p className="text-sm text-muted-foreground">
                  Please select a file or enter a message to proceed with the transfer.
                </p>
              </div>
            )}
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

export default FileUpload;