import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Spinner } from "@/components/ui/spinner";
import { FileText, RefreshCw, Clock, Send, Archive } from "lucide-react";

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

interface AdminFileInfoProps {
  files: FileInfo[];
  loading: boolean;
  onRefresh: () => void;
}

export function AdminFileInfo({ files, loading, onRefresh }: AdminFileInfoProps) {
  const formatDateTime = (dateString: string) => {
    if (!dateString) return "N/A";
    try {
      return new Date(dateString).toLocaleString();
    } catch {
      return dateString;
    }
  };

  return (
    <div className="w-full max-w-6xl mx-auto">
      <Card className="border-2 border-border shadow-md">
        <CardHeader className="border-b-2 border-border pb-6">
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2 text-xl">
                <FileText className="h-5 w-5 text-primary" />
                <span>All File Transfers</span>
              </CardTitle>
              <CardDescription className="text-base">
                View history of every file transfer recorded by the Admin server
              </CardDescription>
            </div>
            <Button
              onClick={onRefresh}
              disabled={loading}
              variant="outline"
              size="sm"
              className="flex items-center gap-2"
            >
              <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
              Refresh
            </Button>
          </div>
        </CardHeader>
        <CardContent className="p-6">
          {loading && files.length === 0 ? (
            <div className="flex items-center justify-center py-12">
              <Spinner className="h-8 w-8" />
              <span className="ml-2">Loading file information...</span>
            </div>
          ) : files.length === 0 ? (
            <div className="text-center py-12">
              <Archive className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <h3 className="text-lg font-medium text-muted-foreground mb-2">No files found</h3>
              <p className="text-sm text-muted-foreground">No file transfer history available.</p>
            </div>
          ) : (
            <div className="space-y-4">
              {files.map((file, index) => (
                <Card key={`${String(file._id)}-${index}`} className="border-2 border-border shadow-sm">
                  <CardContent className="p-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                      <div className="space-y-2">
                        <div className="flex items-center gap-2">
                          <FileText className="h-4 w-4 text-primary" />
                          <span className="font-medium">File Details</span>
                        </div>
                        <div className="text-sm space-y-1">
                          <p><span className="font-medium">Name:</span> {file.name}</p>
                          <p><span className="font-medium">Path:</span> {file.path}</p>
                        </div>
                      </div>

                      <div className="space-y-2">
                        <div className="flex items-center gap-2">
                          <Send className="h-4 w-4 text-blue-500" />
                          <span className="font-medium">Transfer Info</span>
                        </div>
                        <div className="text-sm space-y-1">
                          <p><span className="font-medium">From:</span> {file.sender_bank_id}</p>
                          <p><span className="font-medium">To:</span> {file.receiver_bank_id}</p>
                          {file.message && (
                            <p><span className="font-medium">Message:</span> {file.message}</p>
                          )}
                        </div>
                      </div>

                      <div className="space-y-2">
                        <div className="flex items-center gap-2">
                          <Clock className="h-4 w-4 text-green-500" />
                          <span className="font-medium">Timestamps</span>
                        </div>
                        <div className="text-sm space-y-1">
                          <p><span className="font-medium">Sent:</span> {formatDateTime(file.time_sent_at)}</p>
                          <p><span className="font-medium">Received:</span> {formatDateTime(file.time_received_at)}</p>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default AdminFileInfo; 