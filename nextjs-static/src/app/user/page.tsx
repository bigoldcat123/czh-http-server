'use client'
import Link from "next/link";
import { useSearchParams } from "next/navigation";
export default function UserSearch() {
    const search = useSearchParams()
    const id = search.get('id')

  return (
    <>
    {id}
      <Link href="/">home</Link>
    </>
  );
}