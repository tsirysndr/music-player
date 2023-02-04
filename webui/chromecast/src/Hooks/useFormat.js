export const useTimeFormat = () => {
  const formatTime = (millis) => {
    let minutes = Math.floor(millis / 60000);
    const seconds = ((millis % 60000) / 1000).toFixed(0);
    const secondsDisplay = seconds.length === 1 ? `0${seconds}` : seconds;

    if (seconds === "60") {
      minutes += 1;
      minutes = minutes < 10 ? `0${minutes}` : minutes;
      return `${minutes}:00`;
    } else {
      minutes = minutes < 10 ? `0${minutes}` : minutes;
      return `${minutes}:${secondsDisplay}`;
    }
  };
  return { formatTime };
};
