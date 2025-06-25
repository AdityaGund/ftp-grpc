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
  const [selectedBanks, setSelectedBanks] = useState<Bank[]>([]);
  const [uploading, setUploading] = useState<boolean>(false);
  const [banks, setBanks] = useState<Bank[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const { username, user } = useAuth();
  const token = localStorage.getItem("jwt");
  const [dropdownOpen, setDropdownOpen] = useState(false);

  useEffect(() => {
    fetchAvailableBanks(token)
      .then((response) => {
        setBanks(response.data);
      })
      .catch(() => {
        toast.error("Failed to load available banks.");
      });
  }, []);

  const handleBanksChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const selectedOptions = Array.from(e.target.selectedOptions).map(opt => opt.value);
    const selected = banks.filter(b => selectedOptions.includes(b.username));
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

      // Show overall message
      toast.success(response.data.message);

      // Show per-destination results
      if (response.data.results && Array.isArray(response.data.results)) {
        response.data.results.forEach((result: any) => {
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
      toast.error("Transfer failed. Please try again.");
    } finally {
      setUploading(false);
  }
    // try {
    //   // Send array of banks to backend
    //   const response = await uploadFile(
    //     file,
    //     message || null,
    //     selectedBanks.map(b => ({ username: b.username, ip: b.ip })), // array of banks
    //     username,
    //     token,
    //     user?.role === 'admin' ? 'admin' : 'bank',
    //     // (pct) => setProgress(pct)
    //   );

    //   toast.success("Transfer completed successfully!");
    //   setFile(null);
    //   setMessage("");
    //   setSelectedBanks([]);
    //   if (fileInputRef.current) {
    //     fileInputRef.current.value = "";
    //   }
    // } catch (error) {
    //   toast.error("Transfer failed. Please try again.");
    // } finally {
    //   setUploading(false);
    // }
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

  const selectAllBanks = () => setSelectedBanks(banks);

  // Deselect all banks
  const deselectAllBanks = () => setSelectedBanks([]);
  const toggleBank = (bank: Bank) => {
    setSelectedBanks((prev) =>
      prev.some((b) => b.username === bank.username)
        ? prev.filter((b) => b.username !== bank.username)
        : [...prev, bank]
    );
  };
  // Close dropdown on outside click
  const dropdownRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setDropdownOpen(false);
      }
    }
    if (dropdownOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [dropdownOpen]);

  return (
    <div className="w-full max-w-3xl mx-auto">
      <Card className="border shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="border-b border-border/50 pb-6">
          <CardTitle className="flex items-center gap-3 text-2xl font-bold">
            <div className="p-2 bg-primary/10 rounded-lg">
              <Upload className="h-6 w-6 text-primary" />
            </div>
            <span>Secure File Transfer</span>
          </CardTitle>
          <CardDescription className="text-base leading-relaxed">
            Transfer files or messages securely via encrypted gRPC connection
          </CardDescription>
        </CardHeader>
        <CardContent className="p-8">
          <form onSubmit={handleSubmit} className="space-y-8">
            <div className="space-y-4">
              <label className="text-sm font-semibold flex items-center gap-2 cursor-pointer">
                <User className="h-4 w-4 text-muted-foreground" />
                <span>Destination Banks</span>
              </label>
              <div className="relative" ref={dropdownRef}>
                <button
                  type="button"
                  className="w-full p-3 border rounded-lg bg-background text-left flex justify-between items-center hover:border-ring/60 focus:border-ring transition-all duration-200"
                  onClick={() => setDropdownOpen((open) => !open)}
                  disabled={uploading}
                >
                  <span>
                    {selectedBanks.length === 0
                      ? "Select destination banks..."
                      : `${selectedBanks.length} selected`}
                  </span>
                  <svg className={`w-4 h-4 ml-2 transition-transform ${dropdownOpen ? "rotate-180" : ""}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                  </svg>
                </button>
                {dropdownOpen && (
                  <div className="absolute z-10 mt-2 w-full bg-white border rounded-lg shadow-lg max-h-60 overflow-y-auto">
                    <div className="flex items-center px-3 py-2 border-b">
                      <button
                        type="button"
                        className="text-xs text-primary mr-2 underline"
                        onClick={selectAllBanks}
                        disabled={banks.length === 0}
                      >
                        Select All
                      </button>
                      <button
                        type="button"
                        className="text-xs text-muted-foreground underline"
                        onClick={deselectAllBanks}
                        disabled={selectedBanks.length === 0}
                      >
                        Clear
                      </button>
                    </div>
                    {banks.map((bank) => (
                      <label key={bank.username} className="flex items-center px-4 py-2 cursor-pointer hover:bg-muted/30">
                        <input
                          type="checkbox"
                          checked={selectedBanks.some((b) => b.username === bank.username)}
                          onChange={() => toggleBank(bank)}
                          className="mr-2"
                          disabled={uploading}
                        />
                        <span>{bank.username} <span className="text-xs text-muted-foreground">({bank.ip})</span></span>
                      </label>
                    ))}
                    {banks.length === 0 && (
                      <div className="px-4 py-2 text-sm text-muted-foreground">No banks available</div>
                    )}
                  </div>
                )}
              </div>
              {/* Show selected banks as chips */}
              {selectedBanks.length > 0 && (
                <div className="flex flex-wrap gap-2 mt-2">
                  {selectedBanks.map((b) => (
                    <span
                      key={b.username}
                      className="flex items-center bg-primary/10 text-primary px-3 py-1 rounded-full text-xs font-medium border"
                    >
                      {b.username}
                      <button
                        type="button"
                        className="ml-2 text-primary hover:text-destructive"
                        onClick={() => toggleBank(b)}
                        disabled={uploading}
                        aria-label={`Remove ${b.username}`}
                      >
                        ×
                      </button>
                    </span>
                  ))}
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
