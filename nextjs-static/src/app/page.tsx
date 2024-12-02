'use client'
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";

export default function Home() {
  const router = useRouter();
  const [state, setState] = useState('');
  return (
    <div>
      hello i am home
      <div>
        <Link href="/dashboard">dashboard</Link>
        <input type="text" onChange={(e) => setState(e.target.value)} className=" outline outline-2 outline-blue-400" />
        <button
        className="outline outline-2 outline-blue-400"
          onClick={() => router.push(`/user?id=${state}`)}
        >seaech!</button>
      </div>
    </div>
  );
}
