import { useEffect } from "react";
import { useNavigate } from "react-router";

export default function HomePage() {
  const navigate = useNavigate();

  useEffect(() => {
    navigate("/browse", { replace: true });
  }, [navigate]);

  return (
    <div className="flex items-center justify-center h-screen">
      <p>Redirecting to file browser...</p>
    </div>
  );
}
