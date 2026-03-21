import React, { useEffect, useRef } from 'react';

interface FrequencyVisualizerProps {
  isPlaying: boolean;
  audioUrl?: string;
}

const FrequencyVisualizer: React.FC<FrequencyVisualizerProps> = ({ isPlaying, audioUrl }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyzerRef = useRef<AnalyserNode | null>(null);
  const requestRef = useRef<number>();

  useEffect(() => {
    if (!isPlaying) {
      if (requestRef.current) cancelAnimationFrame(requestRef.current);
      return;
    }

    const initAudio = async () => {
      if (!audioContextRef.current) {
        audioContextRef.current = new (window.AudioContext || (window as any).webkitAudioContext)();
        analyzerRef.current = audioContextRef.current.createAnalyser();
        analyzerRef.current.fftSize = 256;
      }

      const draw = () => {
        if (!canvasRef.current || !analyzerRef.current) return;
        
        const canvas = canvasRef.current;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const bufferLength = analyzerRef.current.frequencyBinCount;
        const dataArray = new Uint8Array(bufferLength);
        analyzerRef.current.getByteFrequencyData(dataArray);

        ctx.clearRect(0, 0, canvas.width, canvas.height);
        
        const barWidth = (canvas.width / bufferLength) * 2.5;
        let barHeight;
        let x = 0;

        for (let i = 0; i < bufferLength; i++) {
          barHeight = (dataArray[i] / 255) * canvas.height;

          // Gradient for "Monster WAV" feel
          const gradient = ctx.createLinearGradient(0, canvas.height, 0, 0);
          gradient.addColorStop(0, '#7c3aed'); // Primary violet
          gradient.addColorStop(1, '#a78bfa'); // Lighter violet

          ctx.fillStyle = gradient;
          ctx.fillRect(x, canvas.height - barHeight, barWidth, barHeight);

          x += barWidth + 1;
        }

        requestRef.current = requestAnimationFrame(draw);
      };

      draw();
    };

    initAudio();

    return () => {
      if (requestRef.current) cancelAnimationFrame(requestRef.current);
    };
  }, [isPlaying]);

  return (
    <div className="w-full h-32 bg-zinc-950/50 rounded-lg overflow-hidden border border-zinc-800/50 flex flex-col">
      <div className="px-3 py-1 flex justify-between items-center border-b border-zinc-800/50 bg-zinc-900/30">
        <span className="text-[10px] font-mono text-primary uppercase tracking-widest font-bold">Monster WAV Monitor</span>
        <div className="flex gap-1">
          <div className={`w-1 h-1 rounded-full ${isPlaying ? 'bg-green-500 animate-pulse' : 'bg-zinc-700'}`} />
          <div className="w-1 h-1 rounded-full bg-zinc-700" />
        </div>
      </div>
      <canvas 
        ref={canvasRef} 
        className="w-full flex-1"
        width={400} 
        height={100}
      />
    </div>
  );
};

export default FrequencyVisualizer;
