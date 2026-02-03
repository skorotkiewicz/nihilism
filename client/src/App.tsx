import { Save, Volume2, VolumeX } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import "./App.css";

// Types matching the Rust backend
interface Choice {
	id: string;
	text: string;
	consequence_hint: string | null;
}

interface NarrativeMoment {
	id: string;
	text: string;
	speaker: string | null;
	mood: string;
	choices: Choice[];
	timestamp: string;
}

interface Loop {
	number: number;
	started_at: string;
	ended_at: string | null;
	choices_made: string[];
	outcome: string | null;
}

interface PersistentMemory {
	total_loops: number;
	total_choices: number;
	dark_choices: number;
	light_choices: number;
	key_memories: string[];
	character_deaths: Record<string, number>;
	truths_discovered: string[];
	nihilism_score: number;
}

interface Player {
	id: string;
	name: string | null;
	current_loop: Loop;
	memory: PersistentMemory;
	narrative_history: NarrativeMoment[];
	created_at: string;
}

interface EndingResponse {
	ending_type: string;
	title: string;
	description: string;
	total_loops: number;
	total_choices: number;
	nihilism_score: number;
	dark_choices: number;
	light_choices: number;
}

interface NarrativeResponse {
	moment: NarrativeMoment;
	loop_number: number;
	nihilism_score: number;
	ending: EndingResponse | null;
}

const API_BASE = "/api";

// Ambient music URLs (royalty-free atmospheric tracks)
const AMBIENT_TRACKS = {
	neutral: "/audio/neutral.mp3",
	hopeful: "/audio/hopeful.mp3",
	dark: "/audio/dark.mp3",
	transcendent: "/audio/transcendent.mp3",
};

// Portrait images based on mood
const PORTRAITS: Record<string, string> = {
	neutral: "/portraits/neutral.png",
	hopeful: "/portraits/hopeful.png",
	nihilistic: "/portraits/nihilistic.png",
	dark: "/portraits/dark.png",
	transcendent: "/portraits/transcendent.png",
};

function App() {
	const [player, setPlayer] = useState<Player | null>(null);
	const [currentMoment, setCurrentMoment] = useState<NarrativeMoment | null>(
		null,
	);
	const [narrativeHistory, setNarrativeHistory] = useState<NarrativeMoment[]>(
		[],
	);
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [showMemory, setShowMemory] = useState(true);
	const [ending, setEnding] = useState<EndingResponse | null>(null);
	const [audioEnabled, setAudioEnabled] = useState(false);
	const [currentTrack, setCurrentTrack] = useState<string>("neutral");
	const [savedSaves, setSavedSaves] = useState<string[]>([]);

	const audioRef = useRef<HTMLAudioElement | null>(null);

	// Fetch saves on mount
	const fetchSaves = useCallback(async () => {
		try {
			const response = await fetch(`${API_BASE}/game/list`);
			if (response.ok) {
				const data = await response.json();
				setSavedSaves(data.saves);
			}
		} catch (err) {
			console.error("Failed to fetch saves", err);
		}
	}, []);

	useEffect(() => {
		fetchSaves();
	}, [fetchSaves]);

	// Auto-load last session if exists
	useEffect(() => {
		const lastPlayerId = localStorage.getItem("nihilism_player_id");
		if (lastPlayerId && !player) {
			// Optional: auto-load or just show in UI
		}
	}, [player]);

	// Initialize audio
	useEffect(() => {
		audioRef.current = new Audio();
		audioRef.current.loop = true;
		audioRef.current.volume = 0.3;

		return () => {
			if (audioRef.current) {
				audioRef.current.pause();
				audioRef.current = null;
			}
		};
	}, []);

	// Update ambient music based on nihilism score
	useEffect(() => {
		if (!player || !audioRef.current) return;

		const score = player.memory.nihilism_score;
		let newTrack = "neutral";

		if (score >= 50) {
			newTrack = "dark";
		} else if (score >= 20) {
			newTrack = "neutral";
		} else if (score <= -50) {
			newTrack = "transcendent";
		} else if (score <= -20) {
			newTrack = "hopeful";
		}

		if (newTrack !== currentTrack) {
			setCurrentTrack(newTrack);
			if (audioEnabled && audioRef.current) {
				audioRef.current.src =
					AMBIENT_TRACKS[newTrack as keyof typeof AMBIENT_TRACKS];
				audioRef.current.play().catch(() => {});
			}
		}
	}, [player, audioEnabled, currentTrack]);

	// Toggle audio
	const toggleAudio = useCallback(() => {
		if (!audioRef.current) return;

		if (audioEnabled) {
			audioRef.current.pause();
			setAudioEnabled(false);
		} else {
			audioRef.current.src =
				AMBIENT_TRACKS[currentTrack as keyof typeof AMBIENT_TRACKS];
			audioRef.current.play().catch(() => {});
			setAudioEnabled(true);
		}
	}, [audioEnabled, currentTrack]);

	// Get portrait based on mood/nihilism
	const getPortrait = useCallback(
		(mood: string, nihilismScore: number): string => {
			if (nihilismScore >= 50) return PORTRAITS.nihilistic;
			if (nihilismScore <= -50) return PORTRAITS.hopeful;

			const moodLower = mood.toLowerCase();
			if (moodLower === "hopeful") return PORTRAITS.hopeful;
			if (moodLower === "nihilistic" || moodLower === "dark")
				return PORTRAITS.nihilistic;
			if (moodLower === "transcendent") return PORTRAITS.transcendent;
			return PORTRAITS.neutral;
		},
		[],
	);

	// Start a new game
	const startNewGame = useCallback(async () => {
		setLoading(true);
		setError(null);
		setEnding(null);

		try {
			const response = await fetch(`${API_BASE}/game/new`, {
				method: "POST",
			});

			if (!response.ok) throw new Error("Failed to start game");

			const data = await response.json();
			setPlayer(data.player);
			localStorage.setItem("nihilism_player_id", data.player.id);
			fetchSaves();

			// Now start the narrative
			const narrativeResponse = await fetch(
				`${API_BASE}/game/${data.player.id}/start`,
				{
					method: "POST",
				},
			);

			if (!narrativeResponse.ok) throw new Error("Failed to start narrative");

			const narrativeData: NarrativeResponse = await narrativeResponse.json();
			setCurrentMoment(narrativeData.moment);
			setNarrativeHistory([narrativeData.moment]);

			if (narrativeData.ending) {
				setEnding(narrativeData.ending);
			}
		} catch (err) {
			setError(err instanceof Error ? err.message : "Unknown error occurred");
		} finally {
			setLoading(false);
		}
	}, [fetchSaves]);

	// Make a choice
	const makeChoice = useCallback(
		async (choice: Choice) => {
			if (!player) return;

			setLoading(true);
			setError(null);

			try {
				const response = await fetch(`${API_BASE}/game/${player.id}/choice`, {
					method: "POST",
					headers: {
						"Content-Type": "application/json",
					},
					body: JSON.stringify({
						choice_id: choice.id,
						choice_text: choice.text,
					}),
				});

				if (!response.ok) throw new Error("Failed to process choice");

				const data: NarrativeResponse = await response.json();
				setCurrentMoment(data.moment);
				setNarrativeHistory((prev) => [...prev, data.moment]);

				// Update player's nihilism score
				setPlayer((prev) =>
					prev
						? {
								...prev,
								memory: {
									...prev.memory,
									nihilism_score: data.nihilism_score,
									total_choices: prev.memory.total_choices + 1,
								},
								current_loop: {
									...prev.current_loop,
									number: data.loop_number,
								},
							}
						: null,
				);

				// Check for ending
				if (data.ending) {
					setEnding(data.ending);
				}
			} catch (err) {
				setError(err instanceof Error ? err.message : "Unknown error occurred");
			} finally {
				setLoading(false);
			}
		},
		[player],
	);

	// Reset the loop
	const resetLoop = useCallback(async () => {
		if (!player) return;

		setLoading(true);
		setError(null);

		try {
			const response = await fetch(`${API_BASE}/game/${player.id}/reset`, {
				method: "POST",
			});

			if (!response.ok) throw new Error("Failed to reset loop");

			const data = await response.json();
			setPlayer(data.player);
			setNarrativeHistory([]);
			setCurrentMoment(null);
			setEnding(null);

			// Start new narrative
			const narrativeResponse = await fetch(
				`${API_BASE}/game/${data.player.id}/start`,
				{
					method: "POST",
				},
			);

			if (!narrativeResponse.ok) throw new Error("Failed to start narrative");

			const narrativeData: NarrativeResponse = await narrativeResponse.json();
			setCurrentMoment(narrativeData.moment);
			setNarrativeHistory([narrativeData.moment]);
		} catch (err) {
			setError(err instanceof Error ? err.message : "Unknown error occurred");
		} finally {
			setLoading(false);
		}
	}, [player]);

	// Load a specific game
	const loadGame = useCallback(async (id: string) => {
		setLoading(true);
		setError(null);
		setEnding(null);

		try {
			const response = await fetch(`${API_BASE}/game/load/${id}`);
			if (!response.ok) throw new Error("Failed to load game");

			const data = await response.json();
			setPlayer(data.player);
			localStorage.setItem("nihilism_player_id", data.player.id);

			// Fetch state to get current moment
			const stateResponse = await fetch(`${API_BASE}/game/${id}`);
			if (stateResponse.ok) {
				const stateData = await stateResponse.json();
				setCurrentMoment(stateData.current_moment);
				setNarrativeHistory(data.player.narrative_history || []);
			}
		} catch (err) {
			setError(err instanceof Error ? err.message : "Unknown error occurred");
		} finally {
			setLoading(false);
		}
	}, []);

	// Save game
	const saveGame = useCallback(async () => {
		if (!player) return;

		try {
			const response = await fetch(`${API_BASE}/game/save/${player.id}`, {
				method: "POST",
			});
			const data = await response.json();
			alert(data.message);
		} catch {
			alert("Failed to save game");
		}
	}, [player]);

	// Get nihilism class for styling
	const getNihilismClass = (score: number): string => {
		if (score > 30) return "dark";
		if (score < -30) return "light";
		return "neutral";
	};

	// Get mood class
	const getMoodClass = (mood: string): string => {
		return `mood-${mood.toLowerCase()}`;
	};

	return (
		<div className="app">
			{/* Header */}
			<header className="header">
				<h1 className="logo">Nihilism</h1>

				<div className="header-controls">
					{player && (
						<div className="stats">
							<div className="stat">
								<span className="stat-label">Loop</span>
								<span className="stat-value loop">
									#{player.current_loop.number}
								</span>
							</div>
							<div className="stat">
								<span className="stat-label">Nihilism</span>
								<span
									className={`stat-value nihilism ${getNihilismClass(player.memory.nihilism_score)}`}
								>
									{player.memory.nihilism_score > 0 ? "+" : ""}
									{player.memory.nihilism_score}
								</span>
							</div>
							<div className="stat">
								<span className="stat-label">Choices</span>
								<span className="stat-value">
									{player.memory.total_choices}
								</span>
							</div>
						</div>
					)}

					<button
						type="button"
						className="audio-toggle"
						onClick={toggleAudio}
						title={audioEnabled ? "Mute" : "Enable ambient music"}
					>
						{audioEnabled ? <Volume2 size={20} /> : <VolumeX size={20} />}
					</button>

					{player && (
						<button
							type="button"
							className="save-button"
							onClick={saveGame}
							title="Save game"
						>
							<Save size={20} />
						</button>
					)}
				</div>
			</header>

			{/* Main Game Area */}
			<main className="game-area">
				{/* Ending Screen */}
				{ending && (
					<div className="ending-screen">
						<div className="ending-content">
							<h2 className="ending-title">{ending.title}</h2>
							<p className="ending-description">{ending.description}</p>

							<div className="ending-stats">
								<div className="ending-stat">
									<span className="ending-stat-label">Total Loops</span>
									<span className="ending-stat-value">
										{ending.total_loops}
									</span>
								</div>
								<div className="ending-stat">
									<span className="ending-stat-label">Total Choices</span>
									<span className="ending-stat-value">
										{ending.total_choices}
									</span>
								</div>
								<div className="ending-stat">
									<span className="ending-stat-label">Final Score</span>
									<span
										className={`ending-stat-value ${getNihilismClass(ending.nihilism_score)}`}
									>
										{ending.nihilism_score > 0 ? "+" : ""}
										{ending.nihilism_score}
									</span>
								</div>
								<div className="ending-stat">
									<span className="ending-stat-label">Dark / Light</span>
									<span className="ending-stat-value">
										{ending.dark_choices} / {ending.light_choices}
									</span>
								</div>
							</div>

							<button
								type="button"
								className="start-button"
								onClick={startNewGame}
							>
								Begin Again
							</button>
						</div>
					</div>
				)}

				{/* Start Screen */}
				{!player && !loading && !ending && (
					<div className="start-screen">
						<h1 className="start-title">Nihilism</h1>
						<p className="start-subtitle">
							A time loop game about the search for meaning in infinite
							repetition. Your choices echo across the void. I remember
							everything.
						</p>
						<button
							type="button"
							className="start-button"
							onClick={startNewGame}
							disabled={loading}
						>
							Enter the Loop
						</button>

						{savedSaves.length > 0 && (
							<div className="saved-games">
								<h3 className="saves-title">I remember these...</h3>
								<div className="saves-list">
									{savedSaves.map((id) => (
										<button
											key={id}
											type="button"
											className="save-item"
											onClick={() => loadGame(id)}
										>
											Continue: {id.substring(0, 8)}
										</button>
									))}
								</div>
							</div>
						)}
					</div>
				)}

				{/* Loading */}
				{loading && (
					<div className="loading">
						<div className="loader"></div>
						<span className="loading-text">The loop iterates...</span>
					</div>
				)}

				{/* Error */}
				{error && (
					<div className="error">
						<p>Something went wrong: {error}</p>
						<button
							type="button"
							className="start-button"
							onClick={() => setError(null)}
						>
							Try Again
						</button>
					</div>
				)}

				{/* Narrative Display */}
				{!loading && currentMoment && !ending && (
					<div className="narrative-container">
						{/* Character Portrait */}
						{player && (
							<div className="portrait-container">
								<img
									src={getPortrait(
										currentMoment.mood,
										player.memory.nihilism_score,
									)}
									alt="Narrator"
									className={`portrait ${getMoodClass(currentMoment.mood)}`}
								/>
							</div>
						)}

						{/* History (collapsed) */}
						{narrativeHistory.length > 1 && (
							<div className="history-container">
								{narrativeHistory.slice(0, -1).map((moment) => (
									<div key={moment.id} className="history-moment">
										{moment.speaker && <strong>{moment.speaker}: </strong>}
										{moment.text.substring(0, 100)}...
									</div>
								))}
							</div>
						)}

						{/* Current Moment */}
						<div
							className={`narrative-moment ${getMoodClass(currentMoment.mood)}`}
						>
							{currentMoment.speaker && (
								<div className="speaker-name">{currentMoment.speaker}</div>
							)}
							<p className="narrative-text">{currentMoment.text}</p>
						</div>

						{/* Choices */}
						<div className="choices-container">
							{currentMoment.choices.map((choice) => (
								<button
									type="button"
									key={choice.id}
									className="choice-button"
									onClick={() => makeChoice(choice)}
									disabled={loading}
								>
									<span className="choice-text">{choice.text}</span>
									{choice.consequence_hint && (
										<span className="choice-hint">
											{choice.consequence_hint}
										</span>
									)}
								</button>
							))}
						</div>

						{/* Reset Loop Button */}
						<button
							type="button"
							className="reset-button"
							onClick={resetLoop}
							disabled={loading}
						>
							Let the loop reset...
						</button>
					</div>
				)}
			</main>

			{/* Memory Panel */}
			{player && !ending && (
				<aside className={`memory-panel ${showMemory ? "" : "hidden"}`}>
					<button
						type="button"
						onClick={() => setShowMemory(!showMemory)}
						className="memory-toggle"
					>
						{showMemory ? "›" : "‹"}
					</button>

					<h3 className="memory-title">Persistent Memory</h3>

					<div className="memory-item">
						<span className="memory-key">Total Loops: </span>
						{player.memory.total_loops}
					</div>

					<div className="memory-item">
						<span className="memory-key">Dark Choices: </span>
						{player.memory.dark_choices}
					</div>

					<div className="memory-item">
						<span className="memory-key">Light Choices: </span>
						{player.memory.light_choices}
					</div>

					{player.memory.key_memories.length > 0 && (
						<>
							<h4
								className="memory-title"
								style={{ marginTop: "var(--space-md)" }}
							>
								I Remember...
							</h4>
							<ul className="memory-list">
								{player.memory.key_memories.slice(-5).map((memory, idx) => (
									<li key={`memory-${idx}-${memory.substring(0, 10)}`}>
										{memory.substring(0, 60)}...
									</li>
								))}
							</ul>
						</>
					)}
				</aside>
			)}

			{/* Footer */}
			<footer className="footer">
				<p className="footer-quote">"Despite everything, it's still you."</p>
				<p>A philosophical time loop experience</p>
			</footer>
		</div>
	);
}

export default App;
