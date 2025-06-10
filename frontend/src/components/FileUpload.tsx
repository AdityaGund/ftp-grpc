import { useState, useRef, type ChangeEvent, type FormEvent } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { toast } from 'sonner';
import { uploadFile } from '@/lib/grpc-client';

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
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>File Transfer</CardTitle>
        <CardDescription>
          Send files or messages securely via gRPC
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-2">
            <label htmlFor="destination" className="text-sm font-medium">
              Destination
            </label>
            <Input
              id="destination"
              placeholder="Enter destination ID"
              value={destination}
              onChange={(e) => setDestination(e.target.value)}
              disabled={uploading}
              required
            />
          </div>
          
          <div className="space-y-2">
            <label htmlFor="file" className="text-sm font-medium">
              File
            </label>
            <Input
              ref={fileInputRef}
              id="file"
              type="file"
              onChange={handleFileChange}
              disabled={uploading}
            />
            {file && (
              <p className="text-xs text-gray-500">
                Selected: {file.name} ({(file.size / 1024).toFixed(2)} KB)
              </p>
            )}
          </div>
          
          <div className="space-y-2">
            <label htmlFor="message" className="text-sm font-medium">
              Message
            </label>
            <Textarea
              id="message"
              placeholder="Enter a message (optional)"
              value={message}
              onChange={(e) => setMessage(e.target.value)}
              disabled={uploading}
              rows={3}
            />
          </div>
          
          {uploading && (
            <div className="space-y-2">
              <label className="text-sm font-medium">Upload Progress</label>
              <Progress value={progress} className="h-2" />
              <p className="text-xs text-right">{progress}%</p>
            </div>
          )}
          
          <div className="flex space-x-2">
            <Button type="submit" disabled={uploading} className="flex-1">
              {uploading ? 'Uploading...' : 'Send'}
            </Button>
            <Button type="button" variant="outline" onClick={handleReset} disabled={uploading}>
              Reset
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
} 