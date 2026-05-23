import init, {greet} from "./pkg/hosho.js";

async function main(){
  await init();

  document.getElementById("greet").addEventListener('click', () => {
    const name = document.getElementById('name').value;
    const output = greet(name);
    document.getElementById('output').textContent = output;
  })
}

main();
