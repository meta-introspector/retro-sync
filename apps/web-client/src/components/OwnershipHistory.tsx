import React from 'react';
import { History, ArrowRight, User, ShieldCheck } from 'lucide-react';

interface TransferEvent {
  from: string;
  to: string;
  timestamp: string;
  type: 'mint' | 'transfer' | 'sale';
  price?: string;
}

interface OwnershipHistoryProps {
  history: TransferEvent[];
}

const OwnershipHistory: React.FC<OwnershipHistoryProps> = ({ history }) => {
  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2 mb-4 text-zinc-400">
        <History className="w-4 h-4" />
        <span className="text-sm font-semibold uppercase tracking-wider">Provenance / History</span>
      </div>

      <div className="relative pl-6 space-y-6 before:absolute before:left-[7px] before:top-2 before:bottom-2 before:w-px before:bg-zinc-800">
        {history.map((event, i) => (
          <div key={i} className="relative group">
            {/* Timeline dot */}
            <div className={`absolute -left-[23px] top-1.5 w-3 h-3 rounded-full border-2 border-background ${
              event.type === 'mint' ? 'bg-primary' : 'bg-zinc-700'
            } group-hover:scale-125 transition-transform`} />

            <div className="flex flex-col gap-1">
              <div className="flex items-center gap-2 text-xs font-mono">
                <span className={`px-1.5 py-0.5 rounded ${
                  event.type === 'mint' ? 'bg-primary/10 text-primary' : 'bg-zinc-800 text-zinc-400'
                } uppercase font-bold text-[9px]`}>
                  {event.type}
                </span>
                <span className="text-zinc-500">{event.timestamp}</span>
              </div>

              <div className="flex items-center gap-2 bg-zinc-900/50 p-2 rounded-lg border border-zinc-800/50">
                <div className="flex items-center gap-1.5 truncate">
                  <User className="w-3 h-3 text-zinc-500" />
                  <span className="text-xs text-zinc-300 font-mono truncate max-w-[100px]">{event.from === '0x0000000000000000000000000000000000000000' ? 'The Void' : event.from}</span>
                </div>
                
                <ArrowRight className="w-3 h-3 text-zinc-600 shrink-0" />

                <div className="flex items-center gap-1.5 truncate">
                  <User className="w-3 h-3 text-primary/70" />
                  <span className="text-xs text-primary font-mono truncate max-w-[100px]">{event.to}</span>
                </div>

                {event.price && (
                  <div className="ml-auto text-xs font-bold text-green-500">
                    {event.price} BTT
                  </div>
                )}
              </div>
            </div>
          </div>
        ))}

        <div className="pt-2 flex items-center gap-2 text-[10px] text-zinc-500">
          <ShieldCheck className="w-3 h-3 text-green-500/50" />
          <span>All transfers verified via BTTC immutable ledger</span>
        </div>
      </div>
    </div>
  );
};

export default OwnershipHistory;
