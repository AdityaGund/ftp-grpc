// import { Button } from "@/components/ui/button";
// import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
// import { Spinner } from "@/components/ui/spinner";
// import { FileText, RefreshCw } from "lucide-react";
// import {
//   Table,
//   TableBody,
//   TableCell,
//   TableHead,
//   TableHeader,
//   TableRow,
// } from "@/components/ui/table";

// interface FileInfo {
//   _id: string | object;
//   name: string;
//   path: string;
//   sender_bank_id: string;
//   receiver_bank_id: string;
//   message: string;
//   time_sent_at: string;
//   time_received_at: string;
// }

// interface FileInfoDisplayProps {
//   files: FileInfo[];
//   loading: boolean;
//   onRefresh: () => void;
//   title: string;
//   description: string;
// }

// export function FileInfoDisplay({ files, loading, onRefresh, title, description }: FileInfoDisplayProps) {
//   const formatDateTime = (dateString: string) => {
//     if (!dateString) return "N/A";
//     try {
//       return new Date(dateString).toLocaleString();
//     } catch {
//       return dateString;
//     }
//   };

//   return (
//     <div className="w-full max-w-6xl mx-auto">
//       <Card className="border-2 border-border shadow-md">
//         <CardHeader className="border-b-2 border-border pb-6">
//           <div className="flex items-center justify-between">
//             <div>
//               <CardTitle className="flex items-center gap-2 text-xl">
//                 <FileText className="h-5 w-5 text-primary" />
//                 <span>{title}</span>
//               </CardTitle>
//               <CardDescription className="text-base">
//                 {description}
//               </CardDescription>
//             </div>
//             <Button 
//               onClick={onRefresh} 
//               disabled={loading}
//               variant="outline"
//               size="sm"
//               className="flex items-center gap-2"
//             >
//               <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
//               Refresh
//             </Button>
//           </div>
//         </CardHeader>
//         <CardContent className="p-6">
//           {loading && files.length === 0 ? (
//             <div className="flex items-center justify-center py-12">
//               <Spinner className="h-8 w-8" />
//               <span className="ml-2">Loading file information...</span>
//             </div>
//           ) : (
//             <div className="rounded-md border">
//               <Table>
//                 <TableHeader>
//                   <TableRow>
//                     <TableHead className="font-medium">Name</TableHead>
//                     <TableHead className="font-medium">From</TableHead>
//                     <TableHead className="font-medium">To</TableHead>
//                     <TableHead className="font-medium">Sent</TableHead>
//                     <TableHead className="font-medium">Received</TableHead>
//                     <TableHead className="font-medium">Message</TableHead>
//                   </TableRow>
//                 </TableHeader>
//                 <TableBody>
//                   {files.length > 0 ? (
//                     files.map((file, index) => (
//                       <TableRow key={`${String(file._id)}-${index}`}>
//                         <TableCell className="font-medium">{file.name}</TableCell>
//                         <TableCell>{file.sender_bank_id}</TableCell>
//                         <TableCell>{file.receiver_bank_id}</TableCell>
//                         <TableCell>{formatDateTime(file.time_sent_at)}</TableCell>
//                         <TableCell>{formatDateTime(file.time_received_at)}</TableCell>
//                         <TableCell className="max-w-xs truncate" title={file.message}>
//                           {file.message}
//                         </TableCell>
//                       </TableRow>
//                     ))
//                   ) : (
//                     <TableRow>
//                       <TableCell
//                         colSpan={6}
//                         className="h-24 text-center text-muted-foreground"
//                       >
//                         No file transfer history available.
//                       </TableCell>
//                     </TableRow>
//                   )}
//                 </TableBody>
//               </Table>
//             </div>
//           )}
//         </CardContent>
//       </Card>
//     </div>
//   );
// }

// export default FileInfoDisplay;

"use client"

import * as React from "react"
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  type SortingState,
  useReactTable,
  type SortingFn,
} from "@tanstack/react-table"
import { ArrowUpDown, FileText, RefreshCw } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Spinner } from "@/components/ui/spinner"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"

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

interface FileInfoDisplayProps {
  files: FileInfo[];
  loading: boolean;
  onRefresh: () => void;
  title: string;
  description: string;
}

// Helper to format date strings
const formatDateTime = (dateString: string) => {
  if (!dateString) return "N/A";
  try {
    return new Date(dateString).toLocaleString();
  } catch {
    return "Invalid Date";
  }
};

// Custom sorting function for date-time strings
const dateTimeSortingFn: SortingFn<FileInfo> = (rowA, rowB, columnId) => {
  const valA = rowA.getValue<string>(columnId);
  const valB = rowB.getValue<string>(columnId);

  // Treat null/empty strings as earliest dates for sorting
  const dateA = valA ? new Date(valA).getTime() : -Infinity;
  const dateB = valB ? new Date(valB).getTime() : -Infinity;

  return dateA - dateB;
};

// Column definitions for the data table
export const columns: ColumnDef<FileInfo>[] = [
  {
    accessorKey: "name",
    header: ({ column }) => (
      <Button
        variant="ghost"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        className="hover:bg-muted/50 font-semibold"
      >
        File Name
        <ArrowUpDown className="ml-2 h-4 w-4" />
      </Button>
    ),
    cell: ({ row }) =>{
      const name = row.getValue("name");
      return (
        <div className="font-medium text-foreground">
          {typeof name === "string" && name.trim() !== "" ? name : "N/A"}
        </div>
      );
    }
  },
  {
    accessorKey: "sender_bank_id",
    header: ({ column }) => (
      <Button
        variant="ghost"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        className="hover:bg-muted/50 font-semibold"
      >
        From
        <ArrowUpDown className="ml-2 h-4 w-4" />
      </Button>
    ),
    cell: ({ row }) => (
      <div className="font-medium text-muted-foreground">
        {row.getValue("sender_bank_id") || "N/A"}
      </div>
    ),
  },
  {
    accessorKey: "receiver_bank_id",
    header: ({ column }) => (
      <Button
        variant="ghost"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        className="hover:bg-muted/50 font-semibold"
      >
        To
        <ArrowUpDown className="ml-2 h-4 w-4" />
      </Button>
    ),
    cell: ({ row }) => (
      <div className="font-medium text-muted-foreground">
        {row.getValue("receiver_bank_id") || "N/A"}
      </div>
    ),
  },
  {
    accessorKey: "time_sent_at",
    sortingFn: dateTimeSortingFn,
    header: ({ column }) => (
      <Button
        variant="ghost"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        className="hover:bg-muted/50 font-semibold"
      >
        Sent At
        <ArrowUpDown className="ml-2 h-4 w-4" />
      </Button>
    ),
    cell: ({ row }) => (
      <div className="text-sm text-muted-foreground">
        {formatDateTime(row.getValue("time_sent_at"))}
      </div>
    ),
  },
  {
    accessorKey: "time_received_at",
    sortingFn: dateTimeSortingFn,
    header: ({ column }) => (
      <Button
        variant="ghost"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        className="hover:bg-muted/50 font-semibold"
      >
        Received At
        <ArrowUpDown className="ml-2 h-4 w-4" />
      </Button>
    ),
    cell: ({ row }) => (
      <div className="text-sm text-muted-foreground">
        {formatDateTime(row.getValue("time_received_at"))}
      </div>
    ),
  },
  {
    accessorKey: "message",
    header: ({ column }) => (
      <Button
        variant="ghost"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
        className="hover:bg-muted/50 font-semibold"
      >
        Message
        <ArrowUpDown className="ml-2 h-4 w-4" />
      </Button>
    ),
    cell: ({ row }) => {
      const message = row.getValue("message") as string;
      return (
        <div className="max-w-xs truncate text-sm text-muted-foreground" title={message}>
          {message || "No message"}
        </div>
      );
    },
  },
];

// The main display component
export function FileInfoDisplay({ files, loading, onRefresh, title, description }: FileInfoDisplayProps) {
  const [sorting, setSorting] = React.useState<SortingState>([
    { id: "time_sent_at", desc: true }
  ])

  const table = useReactTable({
    data: files,
    columns,
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    state: {
      sorting,
    },
  })

  return (
    <div className="w-full mx-auto">
      <Card className="border shadow-lg hover:shadow-xl transition-all duration-300">
        <CardHeader className="border-b border-border/50 pb-6">
          <div className="flex items-center justify-between">
            <div className="space-y-2">
              {/* <CardTitle className="flex items-center gap-3 text-2xl font-bold">
                <div className="p-2 bg-primary/10 rounded-lg">
                  <FileText className="h-6 w-6 text-primary" />
                </div>
                <span>{title}</span>
              </CardTitle> */}
              <CardDescription className="text-base leading-relaxed">
                {description}
              </CardDescription>
            </div>
            <Button
              onClick={onRefresh}
              disabled={loading}
              variant="outline"
              size="sm"
              className="flex items-center gap-2 hover:bg-primary hover:text-primary-foreground transition-all duration-200 font-medium"
            >
              <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
          </div>
        </CardHeader>
        <CardContent className="p-6">
          {loading ? (
            <div className="flex flex-col items-center justify-center py-16 space-y-4">
              <Spinner className="h-8 w-8" />
              <p className="text-muted-foreground font-medium">Loading file information...</p>
            </div>
          ) : (
            <div className="rounded-lg border border-border/50 overflow-hidden">
              <Table>
                <TableHeader className="bg-muted/30">
                  {table.getHeaderGroups().map((headerGroup) => (
                    <TableRow key={headerGroup.id} className="border-b border-border/50">
                      {headerGroup.headers.map((header) => {
                        return (
                          <TableHead key={header.id} className="h-12">
                            {header.isPlaceholder
                              ? null
                              : flexRender(
                                header.column.columnDef.header,
                                header.getContext()
                              )}
                          </TableHead>
                        )
                      })}
                    </TableRow>
                  ))}
                </TableHeader>
                <TableBody>
                  {table.getRowModel().rows?.length ? (
                    table.getRowModel().rows.map((row, index) => (
                      <TableRow
                        key={row.id}
                        data-state={row.getIsSelected() && "selected"}
                        className={`border-b border-border/30 hover:bg-muted/30 transition-colors duration-200 ${
                          index % 2 === 0 ? 'bg-background' : 'bg-muted/10'
                        }`}
                      >
                        {row.getVisibleCells().map((cell) => (
                          <TableCell key={cell.id} className="py-4">
                            {flexRender(
                              cell.column.columnDef.cell,
                              cell.getContext()
                            )}
                          </TableCell>
                        ))}
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell
                        colSpan={columns.length}
                        className="h-32 text-center"
                      >
                        <div className="flex flex-col items-center space-y-2">
                          <FileText className="h-8 w-8 text-muted-foreground" />
                          <p className="text-muted-foreground font-medium">No file transfers found</p>
                          <p className="text-sm text-muted-foreground">Transfer activity will appear here</p>
                        </div>
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default FileInfoDisplay;