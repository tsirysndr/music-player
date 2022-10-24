export const useTimeFormat = () => {
  const formatTime = (millis: number) => {
    let minutes = Math.floor(millis / 60000);
    const seconds = ((millis % 60000) / 1000).toFixed(0);
    const secondsDisplay = seconds.length === 1 ? `0${seconds}` : seconds;

    if (seconds === "60") {
      minutes += 1;
      return `${minutes < 10 ? `0${minutes}` : minutes}:00`;
    } else {
      return `${minutes < 10 ? `0${minutes}` : minutes}:${secondsDisplay}`;
    }
  };
  return { formatTime };
};
