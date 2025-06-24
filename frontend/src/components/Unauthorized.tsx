// src/components/Unauthorized.tsx
"use client";

import React from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Shield, ArrowLeft } from "lucide-react";
import { useNavigate } from "react-router-dom";

const Unauthorized: React.FC = () => {
  const navigate = useNavigate();

  return (
    <div className="min-h-screen bg-gradient-to-br from-background via-background to-muted/30 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <Card className="border shadow-2xl hover:shadow-3xl transition-all duration-300 backdrop-blur-sm">
          <CardHeader className="text-center pb-8 space-y-6">
            <div className="mx-auto w-16 h-16 bg-destructive/10 rounded-2xl flex items-center justify-center shadow-lg hover:shadow-xl transition-all duration-300 hover:scale-105">
              <Shield className="w-8 h-8 text-destructive" />
            </div>
            <div className="space-y-3">
              <CardTitle className="text-3xl font-bold text-destructive tracking-tight">
                Access Denied
              </CardTitle>
              <CardDescription className="text-base leading-relaxed">
                You don't have permission to access this page
              </CardDescription>
            </div>
          </CardHeader>
          <CardContent className="text-center space-y-6">
            <div className="p-4 bg-destructive/5 border border-destructive/20 rounded-lg">
              <p className="text-muted-foreground font-medium">
                Please contact your administrator if you believe this is an error.
              </p>
            </div>
            <Button 
              onClick={() => navigate(-1)}
              className="w-full gap-2 h-11 text-base font-semibold"
              variant="outline"
            >
              <ArrowLeft className="h-5 w-5" />
              Go Back
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default Unauthorized;