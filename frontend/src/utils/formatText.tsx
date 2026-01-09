import React from 'react';

/**
 * Parse text with markdown-style formatting and return React elements
 * Supports: **text** for bold
 */
export function parseFormattedText(text: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  let lastIndex = 0;
  
  // Regex to match **text** for bold
  const boldRegex = /\*\*([^*]+)\*\*/g;
  let match;
  
  while ((match = boldRegex.exec(text)) !== null) {
    // Add text before the match
    if (match.index > lastIndex) {
      parts.push(text.substring(lastIndex, match.index));
    }
    
    // Add the bold text
    parts.push(
      <strong key={match.index} style={{ fontWeight: 'bold' }}>
        {match[1]}
      </strong>
    );
    
    lastIndex = match.index + match[0].length;
  }
  
  // Add remaining text after last match
  if (lastIndex < text.length) {
    parts.push(text.substring(lastIndex));
  }
  
  return parts.length > 0 ? <>{parts}</> : text;
}
