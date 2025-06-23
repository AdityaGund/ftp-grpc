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
      >
        Name
        <ArrowUpDown className="ml-2 h-4 w-4" />
      </Button>
    ),
    cell: ({ row }) =>{
      const name = row.getValue("name");
      return (
        <div className="font-medium">
          {typeof name === "string" && name.trim() !== "" ? name : "N/A"}
        </div>
      );
    }
  },
  //   {
  // accessorKey: "path",
  // header: "Path",
  //   },
  {
    accessorKey: "sender_bank_id",
    header: "From",
  },
  {
    accessorKey: "receiver_bank_id",
    header: "To",
  },
  {
    accessorKey: "time_sent_at",
    sortingFn: dateTimeSortingFn,
    header: ({ column }) => (
      <Button
        variant="ghost"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
      >
        Sent
        <ArrowUpDown className="ml-2 h-4 w-4" />
      </Button>
    ),
    cell: ({ row }) => <div>{formatDateTime(row.getValue("time_sent_at"))}</div>,
  },
  {
    accessorKey: "time_received_at",
    sortingFn: dateTimeSortingFn,
    header: ({ column }) => (
      <Button
        variant="ghost"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
      >
        Received
        <ArrowUpDown className="ml-2 h-4 w-4" />
      </Button>
    ),
    cell: ({ row }) => <div>{formatDateTime(row.getValue("time_received_at"))}</div>,
  },
  {
    accessorKey: "message",
    header: "Message",
    cell: ({ row }) => <div className="truncate max-w-xs">{row.getValue("message")}</div>,
  },
];

// The main display component
export function FileInfoDisplay({ files, loading, onRefresh, title, description }: FileInfoDisplayProps) {
  const [sorting, setSorting] = React.useState<SortingState>([])

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
    <div className="w-full max-w-6xl mx-auto">
      <Card className="border-2 border-border shadow-md">
        <CardHeader className="border-b-2 border-border pb-6">
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2 text-xl">
                <FileText className="h-5 w-5 text-primary" />
                <span>{title}</span>
              </CardTitle>
              <CardDescription className="text-base">
                {description}
              </CardDescription>
            </div>
            <Button
              onClick={onRefresh}
              disabled={loading}
              variant="outline"
              size="sm"
              className="flex items-center gap-2"
            >
              <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
              Refresh
            </Button>
          </div>
        </CardHeader>
        <CardContent className="p-6">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <Spinner className="h-8 w-8" />
              <span className="ml-2">Loading file information...</span>
            </div>
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  {table.getHeaderGroups().map((headerGroup) => (
                    <TableRow key={headerGroup.id}>
                      {headerGroup.headers.map((header) => {
                        return (
                          <TableHead key={header.id}>
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
                    table.getRowModel().rows.map((row) => (
                      <TableRow
                        key={row.id}
                        data-state={row.getIsSelected() && "selected"}
                      >
                        {row.getVisibleCells().map((cell) => (
                          <TableCell key={cell.id}>
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
                        className="h-24 text-center"
                      >
                        No results.
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