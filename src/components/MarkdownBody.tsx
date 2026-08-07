import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

type Props = {
  content: string;
  className?: string;
};

/** Safe Modrinth-style Markdown (no raw HTML). */
export function MarkdownBody({ content, className }: Props) {
  const text = content.trim();
  if (!text) return null;
  return (
    <div className={`euml-md ${className ?? ""}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          a: ({ href, children }) => (
            <a href={href} target="_blank" rel="noreferrer noopener">
              {children}
            </a>
          ),
          img: ({ src, alt }) => (
            <img src={src} alt={alt ?? ""} loading="lazy" className="euml-md-img" />
          ),
        }}
      >
        {text}
      </ReactMarkdown>
    </div>
  );
}
