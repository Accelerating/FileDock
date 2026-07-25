import { Link, useLocation } from "react-router";
import {
  FolderOpen,
  Search,
  Settings,
  HardDrive,
  Activity,
} from "lucide-react";
import { cn } from "~/lib/utils";
import { Button } from "~/components/ui/button";
import { Separator } from "~/components/ui/separator";
import { ScrollArea } from "~/components/ui/scroll-area";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "~/components/ui/tooltip";

const navItems = [
  {
    title: "Files",
    icon: FolderOpen,
    href: "/browse",
  },
  {
    title: "Search",
    icon: Search,
    href: "/search",
  },
  {
    title: "Storage",
    icon: HardDrive,
    href: "/storage",
  },
];

export function Sidebar() {
  const location = useLocation();

  return (
    <aside className="flex flex-col w-64 border-r bg-muted/30">
      {/* Logo */}
      <div className="flex items-center gap-2 p-4 h-16">
        <Activity className="size-6 text-primary" />
        <span className="font-semibold text-lg">FileDock</span>
      </div>

      <Separator />

      {/* Navigation */}
      <ScrollArea className="flex-1 p-2">
        <nav className="flex flex-col gap-1">
          {navItems.map((item) => {
            const isActive =
              location.pathname === item.href ||
              location.pathname.startsWith(item.href + "/");

            return (
              <Tooltip key={item.href}>
                <TooltipTrigger asChild>
                  <Button
                    variant={isActive ? "secondary" : "ghost"}
                    className="w-full justify-start gap-3 px-3 h-10"
                    asChild
                  >
                    <Link to={item.href} className="flex items-center">
                      <item.icon className="size-4 shrink-0" />
                      <span>{item.title}</span>
                    </Link>
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="right">{item.title}</TooltipContent>
              </Tooltip>
            );
          })}
        </nav>
      </ScrollArea>

      <Separator />

      {/* Settings */}
      <div className="p-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" className="w-full justify-start gap-3 px-3 h-10" asChild>
              <Link to="/settings" className="flex items-center">
                <Settings className="size-4 shrink-0" />
                <span>Settings</span>
              </Link>
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right">Settings</TooltipContent>
        </Tooltip>
      </div>
    </aside>
  );
}
