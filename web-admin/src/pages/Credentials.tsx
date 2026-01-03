import { CredentialsList } from "@/features/credentials/CredentialsList";
import { CredentialUpload } from "@/features/credentials/CredentialUpload";

export default function Credentials() {
  return (
    <div className="flex-1 space-y-4 p-8 pt-6">
      <div className="flex items-center justify-between space-y-2">
        <h2 className="text-3xl font-bold tracking-tight">凭据管理</h2>
      </div>
      
      <div className="grid gap-4 md:grid-cols-1 lg:grid-cols-3">
        <div className="lg:col-span-2">
            <CredentialsList />
        </div>
        <div>
            <CredentialUpload />
        </div>
      </div>
    </div>
  );
}
