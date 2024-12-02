import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */
  output:"export",
  // assetPrefix: '/file', // 使静态资源路径相对
  basePath:'/file',
  distDir: 'dist',
  // trailingSlash: true, // 生成以斜杠结尾的路径，确保引用文件正确
};

export default nextConfig;
