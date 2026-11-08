"use strict";

const note = document.getElementById("note");
const count = document.getElementById("count");

function updateCount() {
  count.textContent = `${Array.from(note.value).length} 文字`;
}

note.addEventListener("input", updateCount);
document.getElementById("clear").addEventListener("click", () => {
  note.value = "";
  updateCount();
});
updateCount();

const cards = document.getElementById("cards");
for (let i = 1; i <= 100; i += 1) {
  const card = document.createElement("article");
  card.className = "card";
  const title = document.createElement("h2");
  title.textContent = `項目 ${i}`;
  const description = document.createElement("p");
  description.textContent = "HTML / CSSで描くローカルUI。スクロールと文字表示を確認します。";
  card.append(title, description);
  cards.append(card);
}
