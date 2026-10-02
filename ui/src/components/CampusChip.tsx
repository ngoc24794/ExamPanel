import * as React from 'react'
import { getCampusColorStyle } from '@/lib/theme/campus-colors'

interface CampusChipProps {
  name: string
  color: string
  className?: string
}

export const CampusChip: React.FC<CampusChipProps> = ({
  name,
  color,
  className = '',
}) => {
  const style = getCampusColorStyle(color)

  return (
    <span
      style={style}
      className={`inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium border border-solid transition-colors ${className}`}
    >
      <span
        className="w-1.5 h-1.5 rounded-full shrink-0"
        style={{ backgroundColor: style.color }}
      />
      <span className="truncate max-w-[120px]">{name}</span>
    </span>
  )
}
