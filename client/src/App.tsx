import { useCallback, useState } from "react";
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

interface NarrativeResponse {
	moment: NarrativeMoment;
	loop_number: number;
	nihilism_score: number;
}

const API_BASE = "/api";

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

	// Start a new game
	const startNewGame = useCallback(async () => {
		setLoading(true);
		setError(null);

		try {
			const response = await fetch(`${API_BASE}/game/new`, {
				method: "POST",
			});

			if (!response.ok) throw new Error("Failed to start game");

			const data = await response.json();
			setPlayer(data.player);

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
		} catch (err) {
			setError(err instanceof Error ? err.message : "Unknown error occurred");
		} finally {
			setLoading(false);
		}
	}, []);

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
								},
								current_loop: {
									...prev.current_loop,
									number: data.loop_number,
								},
							}
						: null,
				);
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
							<span className="stat-value">{player.memory.total_choices}</span>
						</div>
					</div>
				)}
			</header>

			{/* Main Game Area */}
			<main className="game-area">
				{/* Start Screen */}
				{!player && !loading && (
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
				{!loading && currentMoment && (
					<div className="narrative-container">
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
			{player && (
				<aside className={`memory-panel ${showMemory ? "" : "hidden"}`}>
					<button
						type="button"
						onClick={() => setShowMemory(!showMemory)}
						style={{
							position: "absolute",
							left: "-24px",
							top: "50%",
							transform: "translateY(-50%)",
							background: "var(--void-surface)",
							border: "1px solid var(--void-border)",
							color: "var(--text-muted)",
							padding: "8px 4px",
							cursor: "pointer",
							fontSize: "0.8rem",
						}}
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
