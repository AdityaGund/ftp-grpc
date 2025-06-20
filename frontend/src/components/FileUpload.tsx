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
  const [uploading, setUploading] = useState<boolean>(false);
  // const [progress, setProgress] = useState<number>(0);
  const [banks, setBanks] = useState<Bank[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { username } = useAuth();
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

  const handleDestinationChange = (value: string) => {
    setDestination(value);
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (!destination) {
      toast.error("Please enter a destination");
      return;
    }

    if (!file && !message) {
      toast.error("Please upload a file or enter a message");
      return;
    }

    setUploading(true);
    // setProgress(0);

    try {
      // setProgress(0);
      const response = await uploadFile(
        file,
        message || null,
        destination,
        username,
        token,
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
    // setProgress(0);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  };

  return (
    <div className="w-full max-w-2xl mx-auto">
      <Card className="border-2 border-border shadow-md">
        <CardHeader className="border-b-2 border-border pb-6">
          <CardTitle className="flex items-center gap-2 text-xl">
            <Upload className="h-5 w-5 text-primary" />
            <span>File Transfer</span>
          </CardTitle>
          <CardDescription className="text-base">
            Transfer files or messages securely via gRPC
          </CardDescription>
        </CardHeader>
        <CardContent className="p-6">
          <form onSubmit={handleSubmit} className="space-y-6">
            <div className="space-y-3">
              <label htmlFor="destination" className="text-sm font-medium flex items-center gap-2">
                <User className="h-4 w-4 text-muted-foreground" />
                <span>Destination</span>
              </label>
              <div className="relative">
                <Input
                  id="destination"
                  list="destinations"
                  placeholder="Select or type destination IP"
                  value={destination}
                  onChange={(e) => handleDestinationChange(e.target.value)}
                  disabled={uploading}
                  required
                  className="bg-background"
                />
                <datalist id="destinations">
                  {banks.map((bank) => (
                    <option
                      key={bank.username}
                      value={bank.ip}
                    >
                      {`${bank.username}`}
                    </option>
                  ))}
                </datalist>
              </div>
            </div>

            <div className="space-y-3">
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
                <p className="text-sm text-muted-foreground bg-muted/50 p-2 rounded border-2 border-border">
                  Selected: <span className="font-medium">{file.name}</span> ({(file.size / 1024).toFixed(2)} KB)
                </p>
              )}
            </div>

            <div className="space-y-3">
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
                rows={4}
                className="bg-background resize-none"
              />
            </div>

            {/* {uploading && (
              <div className="space-y-3 p-4 bg-muted/30 rounded-lg border">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Spinner size="sm" />
                    <span className="text-sm font-medium">Uploading...</span>
                  </div>
                  <span className="text-sm text-muted-foreground">{progress}%</span>
                </div>
                <div className="text-xs text-muted-foreground">
                  Please wait while your file is being transferred securely
                </div>
              </div>
            )} */}

            <div className="flex flex-col sm:flex-row gap-3 pt-4">
              <Button
                type="submit"
                disabled={uploading}
                className="flex-1 gap-2"
              >
                {uploading ? (
                  <>
                    <Spinner size="sm" />
                    Uploading...
                  </>
                ) : (
                  <>
                    Send
                    <SendHorizontal className="h-4 w-4" />
                  </>
                )}
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
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}

export default FileUpload;