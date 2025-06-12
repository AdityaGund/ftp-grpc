/* eslint-disable @typescript-eslint/no-unused-vars */
import { useState, useRef, type ChangeEvent, type FormEvent } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { toast } from 'sonner';
import { uploadFile } from '@/lib/api';
import { Upload, User, SendHorizontal, X, RotateCcw } from 'lucide-react';

export function FileUpload() {
  const [file, setFile] = useState<File | null>(null);
  const [message, setMessage] = useState<string>('');
  const [destination, setDestination] = useState<string>('');
  const [uploading, setUploading] = useState<boolean>(false);
  const [progress, setProgress] = useState<number>(0);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileChange = (e: ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      setFile(e.target.files[0]);
    }
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    
    if (!destination) {
      toast.error('Please enter a destination');
      return;
    }
    
    if (!file && !message) {
      toast.error('Please upload a file or enter a message');
      return;
    }
    
    setUploading(true);
    
    // Simulate progress 
    const progressInterval = setInterval(() => {
      setProgress((prev) => {
        if (prev >= 95) {
          clearInterval(progressInterval);
          return prev;
        }
        return prev + 5;
      });
    }, 200);
    
    try {
      const response = await uploadFile(file, message || null, destination);
      clearInterval(progressInterval);
      setProgress(100);
      
      toast.success('Transfer completed successfully!');
      console.log('Upload response:', response);
      
      // Reset form
      setFile(null);
      setMessage('');
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    } catch (error) {
      clearInterval(progressInterval);
      setProgress(0);
      toast.error('Transfer failed. Please try again.');
      console.error('Upload error:', error);
    } finally {
      setUploading(false);
    }
  };

  const handleReset = () => {
    setFile(null);
    setMessage('');
    setDestination('');
    setProgress(0);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
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
      <CardContent className="pt-6">
        <form onSubmit={handleSubmit} className="space-y-5">
          <div className="space-y-2">
            <label htmlFor="destination" className="text-sm font-medium flex items-center gap-2">
              <User className="h-4 w-4 text-muted-foreground" />
              <span>Destination</span>
            </label>
            <Input
              id="destination"
              placeholder="Enter destination ID"
              value={destination}
              onChange={(e) => setDestination(e.target.value)}
              disabled={uploading}
              required
              className="bg-background"
            />
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
                onChange={handleFileChange}
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
                    if (fileInputRef.current) fileInputRef.current.value = '';
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
              {uploading ? 'Uploading...' : 'Send'}
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
          </div>
        </form>
      </CardContent>
    </Card>
  );
} 