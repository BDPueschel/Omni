interface FilePreview {
    file_type: string;
    content: string;
    filename: string;
    size: string;
    modified: string;
    extension: string;
}

interface Props {
    preview: FilePreview;
}

export function PreviewPanel({ preview }: Props) {
    return (
        <div class="preview-panel">
            <div class="preview-header">
                <div class="preview-filename">{preview.filename}</div>
                <div class="preview-meta">{preview.size} · {preview.modified} · .{preview.extension}</div>
            </div>
            <div class="preview-content">
                {preview.file_type === "text" && (
                    <pre class="preview-text">{preview.content}</pre>
                )}
                {preview.file_type === "image" && preview.content && (
                    <img src={preview.content} class="preview-image" />
                )}
                {preview.file_type === "image" && !preview.content && (
                    <div class="preview-binary">
                        <div class="preview-binary-icon">IMG</div>
                        <div>Image too large to preview</div>
                    </div>
                )}
                {preview.file_type === "binary" && (
                    <div class="preview-binary">
                        <div class="preview-binary-icon">{preview.extension ? preview.extension.toUpperCase() : "BIN"}</div>
                        <div>Binary file — no preview available</div>
                    </div>
                )}
            </div>
        </div>
    );
}

export type { FilePreview };
