import 'package:rinf/rinf.dart';
import 'src/bindings/bindings.dart';
import 'package:flutter/material.dart';

Future<void> main() async {
  await initializeRust(assignRustSignal);
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
      ),
      home: const MyHomePage(title: 'Flutter Demo Home Page'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  int _counter = 0;

  void _incrementCounter() {
    setState(() {
      _counter++;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,

        title: Text(widget.title),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: <Widget>[
            const Text('You have pushed the button this many times:'),
            Text(
              '$_counter',
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            ElevatedButton(
              onPressed: () async {
                MyPreciousData(
                  inputNumbers: [3, 4, 5],
                  inputString: "Zero-const abstraction",
                ).sendSignalToRust();
              },
              child: Text("Send a Signal from Dart to Rust"),
            ),
            StreamBuilder(
              stream: MyAmazingNumber.rustSignalStream,
              builder: (ctx, snapshot) {
                final signalPack = snapshot.data;
                if (signalPack == null) {
                  return Text("Nothing received yet");
                }
                final myAmazingNumber = signalPack.message;
                final currentNumber = myAmazingNumber.currentNumber;
                return Text(currentNumber.toString());
              },
            ),
            StreamBuilder(
              stream: MyTreasureOutput.rustSignalStream,
              builder: (_, snapshot) {
                final signalPack = snapshot.data;
                if (signalPack == null) {
                  return Text("No Value Yet");
                }
                final myTreasureOutput = signalPack.message;
                final currentNumber = myTreasureOutput.currentValue;
                return Text("$currentNumber");
              },
            ),
            ElevatedButton(
              onPressed: () async {
                MyTreasureInput().sendSignalToRust();
              },
              child: Text("Send the input"),
            ),
          ],
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ),
    );
  }
}
