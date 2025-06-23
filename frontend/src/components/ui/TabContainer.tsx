import React, { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";

interface Tab {
  id: string;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  content: React.ReactNode;
}

interface TabContainerProps {
  tabs: Tab[];
  defaultTab?: string;
  className?: string;
}

export function TabContainer({ tabs, defaultTab, className = "" }: TabContainerProps) {
  const [activeTab, setActiveTab] = useState<string>(defaultTab || tabs[0]?.id || "");

  const activeTabContent = tabs.find(tab => tab.id === activeTab)?.content;

  return (
    <div className={`w-full space-y-4 ${className}`}>
      <Card className="border shadow-md hover:shadow-lg transition-all duration-300">
        <CardContent className="p-4">
          <div className="flex flex-col sm:flex-row gap-2 justify-center">
            {tabs.map((tab) => {
              const IconComponent = tab.icon;
              return (
                <Button
                  key={tab.id}
                  variant={activeTab === tab.id ? "default" : "outline"}
                  onClick={() => setActiveTab(tab.id)}
                  className="flex items-center gap-2 min-w-[140px] justify-center font-medium h-9 text-sm"
                >
                  <IconComponent className="h-4 w-4" />
                  {tab.label}
                </Button>
              );
            })}
          </div>
        </CardContent>
      </Card>

      <div className="transition-all duration-300 ease-in-out">
        {activeTabContent}
      </div>
    </div>
  );
} 