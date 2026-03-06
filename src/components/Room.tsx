import type { FileOffer, Transfer, MemberInfo } from "../types";
import { FileOfferCard } from "./FileOffer";
import { FileSend } from "./FileSend";
import { MemberBar } from "./MemberBar";
import { TransferHistory } from "./TransferHistory";

interface RoomProps {
  roomName: string;
  offers: FileOffer[];
  transfers: Map<string, Transfer>;
  currentUserId: string;
  roomId: string;
  members: MemberInfo[];
  onOfferFile: (roomId: string, filePath: string, description?: string, targetUser?: string) => Promise<void>;
  onRequestFile: (offer: FileOffer) => void;
}

export function Room({
  roomName,
  offers,
  transfers,
  currentUserId,
  roomId,
  members,
  onOfferFile,
  onRequestFile,
}: RoomProps) {
  // Filter offers: show room-wide offers, offers targeted at us, and our own targeted offers
  const visibleOffers = offers.filter(
    (o) =>
      !o.targetUser ||
      o.targetUser === currentUserId ||
      o.senderUserId === currentUserId
  );

  return (
    <div className="flex flex-col h-full">
      {/* Room header */}
      <div className="px-5 py-3 border-b border-white/[0.06] bg-white/[0.01]">
        <h2 className="text-white font-medium text-sm">{roomName}</h2>
      </div>

      {/* Member bar */}
      <MemberBar
        members={members}
        currentUserId={currentUserId}
        onSendToUser={(userId, filePath) =>
          onOfferFile(roomId, filePath, undefined, userId)
        }
      />

      {/* File offers list */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3">
        {visibleOffers.length === 0 ? (
          <div className="text-center py-16">
            <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-white/[0.04] border border-white/[0.06] flex items-center justify-center">
              <svg className="w-7 h-7 text-slate-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
                <polyline points="14,2 14,8 20,8" />
              </svg>
            </div>
            <p className="text-slate-500 text-sm">No files shared yet</p>
            <p className="text-slate-600 text-xs mt-1">
              Share a file or drag and drop to get started
            </p>
          </div>
        ) : (
          visibleOffers.map((offer) => (
            <FileOfferCard
              key={offer.offerId}
              offer={offer}
              transfer={transfers.get(offer.offerId)}
              isOwnOffer={offer.senderUserId === currentUserId}
              onRequest={onRequestFile}
            />
          ))
        )}

        {/* Transfer history */}
        {visibleOffers.length > 0 && (
          <TransferHistory transfers={transfers} roomId={roomId} />
        )}
      </div>

      {/* Send area */}
      <div className="p-4 border-t border-white/[0.06]">
        <FileSend
          onSendFile={async (filePath, desc) => onOfferFile(roomId, filePath, desc)}
        />
      </div>
    </div>
  );
}
