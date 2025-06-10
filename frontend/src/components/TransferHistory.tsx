import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

// Mock data for transfer history
// In a real app, this would be fetched from the backend
const mockTransfers = [
  {
    id: 'transfer-001',
    timestamp: '2024-11-01T10:30:00Z',
    fileName: 'document.pdf',
    destination: 'server-b',
    status: 'SUCCESS',
    size: '1.2 MB'
  },
  {
    id: 'transfer-002',
    timestamp: '2024-11-01T09:15:00Z',
    fileName: 'image.jpg',
    destination: 'server-c',
    status: 'SUCCESS',
    size: '3.5 MB'
  },
  {
    id: 'transfer-003',
    timestamp: '2024-10-31T16:45:00Z',
    message: 'Hello, this is a test message',
    destination: 'server-b',
    status: 'SUCCESS',
    size: '1 KB'
  }
];

export function TransferHistory() {
  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Recent Transfers</CardTitle>
        <CardDescription>
          View your recent file and message transfers
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b">
                <th className="py-3 text-left">Time</th>
                <th className="py-3 text-left">Content</th>
                <th className="py-3 text-left">Destination</th>
                <th className="py-3 text-left">Status</th>
                <th className="py-3 text-left">Size</th>
              </tr>
            </thead>
            <tbody>
              {mockTransfers.map((transfer) => (
                <tr key={transfer.id} className="border-b">
                  <td className="py-3">
                    {new Date(transfer.timestamp).toLocaleString()}
                  </td>
                  <td className="py-3">
                    {transfer.fileName ? (
                      <span className="flex items-center">
                        <svg 
                          className="w-4 h-4 mr-1" 
                          fill="none" 
                          stroke="currentColor" 
                          viewBox="0 0 24 24" 
                          xmlns="http://www.w3.org/2000/svg"
                        >
                          <path 
                            strokeLinecap="round" 
                            strokeLinejoin="round" 
                            strokeWidth={2} 
                            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" 
                          />
                        </svg>
                        {transfer.fileName}
                      </span>
                    ) : (
                      <span className="flex items-center">
                        <svg 
                          className="w-4 h-4 mr-1" 
                          fill="none" 
                          stroke="currentColor" 
                          viewBox="0 0 24 24" 
                          xmlns="http://www.w3.org/2000/svg"
                        >
                          <path 
                            strokeLinecap="round" 
                            strokeLinejoin="round" 
                            strokeWidth={2} 
                            d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" 
                          />
                        </svg>
                        Message
                      </span>
                    )}
                  </td>
                  <td className="py-3">{transfer.destination}</td>
                  <td className="py-3">
                    <span className={`px-2 py-1 rounded-full text-xs ${
                      transfer.status === 'SUCCESS' 
                        ? 'bg-green-100 text-green-800' 
                        : 'bg-red-100 text-red-800'
                    }`}>
                      {transfer.status}
                    </span>
                  </td>
                  <td className="py-3">{transfer.size}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </CardContent>
    </Card>
  );
} 