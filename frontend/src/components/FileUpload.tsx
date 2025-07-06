/* eslint-disable @typescript-eslint/no-unused-vars */
import { useState, useEffect, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardContent } from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { MultiSelect } from "@/components/ui/multi-select";
import { toast } from "sonner";
import { uploadFile, fetchAvailableBanks } from "@/lib/api";
import { getErrorMessage } from "@/lib/utils";
import { Upload, User, SendHorizontal, X, RotateCcw, Building2 } from "lucide-react";
import { useRef } from "react";
import { useAuth } from "@/lib/AuthContext";

interface Bank {
  username: string;
  ip: string;
}

interface TransferResult {
  status: string;
  destination: string;
  destination_ip: string;
  error?: string;
}

export function FileUpload() {
  const [file, setFile] = useState<File | null>(null);
  const [message, setMessage] = useState<string>("");
  const [selectedBanks, setSelectedBanks] = useState<Bank[]>([]);
  const [uploading, setUploading] = useState<boolean>(false);
  const [banks, setBanks] = useState<Bank[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { username, user } = useAuth();
  const token = localStorage.getItem("jwt");

  useEffect(() => {
    fetchAvailableBanks(token)
      .then((response) => {
        setBanks(response.data);
      })
      .catch(() => {
        toast.error("Failed to load available banks.");
      });
  }, []);

  const handleBanksChange = (selectedValues: string[]) => {
    const selected = banks.filter(bank => selectedValues.includes(bank.username));
    setSelectedBanks(selected);
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (selectedBanks.length === 0) {
      toast.error("Please select at least one destination bank");
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
        selectedBanks.map(b => ({ username: b.username, ip: b.ip })),
        username,
        token,
        user?.role === 'admin' ? 'admin' : 'bank',
      );

      // Show overall outcome based on results
      const allSuccess = Array.isArray(response.data.results) && response.data.results.every((r: any) => r.status === "success");
      if (allSuccess) {
        toast.success(response.data.message || "Transfer complete.");
      } else {
        toast.error(response.data.message || "Transfer encountered failures.");
      }

      // Show per-destination results
      if (response.data.results && Array.isArray(response.data.results)) {
        response.data.results.forEach((result: TransferResult) => {
          if (result.status === "success") {
            toast.success(`Transfer to ${result.destination} (${result.destination_ip}) succeeded.`);
          } else {
            toast.error(`Transfer to ${result.destination} (${result.destination_ip}) failed: ${result.error}`);
          }
        });
      }

      setFile(null);
      setMessage("");
      setSelectedBanks([]);
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    } catch (error) {
      toast.error(getErrorMessage(error));
    } finally {
      setUploading(false);
    }
  };

  const handleReset = () => {
    setFile(null);
    setMessage("");
    setSelectedBanks([]);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  };

  const handleFileSelect = () => {
    if (fileInputRef.current) {
      fileInputRef.current.click();
    }
  };

  // Convert banks to options format for MultiSelect
  const bankOptions = banks.map(bank => ({
    label: `${bank.username} (${bank.ip})`,
    value: bank.username,
    icon: Building2
  }));

  return (
    <div className="w-full max-w-3xl mx-auto">
      <Card className="border shadow-lg hover:shadow-xl transition-all duration-300">
        <CardContent className="p-8">
          <form onSubmit={handleSubmit} className="space-y-8">
            <div className="space-y-4">
              <label className="text-sm font-semibold flex items-center gap-2 cursor-pointer">
                <User className="h-4 w-4 text-muted-foreground" />
                <span>Destination Banks</span>
              </label>
              <MultiSelect
                options={bankOptions}
                onValueChange={handleBanksChange}
                defaultValue={selectedBanks.map(bank => bank.username)}
                placeholder="Select destination banks..."
                variant="default"
                animation={2}
                maxCount={3}
                className="w-full"
                disabled={uploading}
              />
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
