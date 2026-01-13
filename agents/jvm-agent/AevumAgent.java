package io.aevum.agent;

import java.lang.instrument.Instrumentation;
import java.lang.instrument.ClassFileTransformer;
import java.security.ProtectionDomain;
import org.objectweb.asm.*;
import org.objectweb.asm.commons.AdviceAdapter;

/**
 * Aevum JVM Agent - Bytecode Instrumentation
 * 
 * This agent uses ASM to instrument Java bytecode for execution tracing.
 * 
 * Usage:
 * java -javaagent:aevum-agent.jar=trace-id=my-trace,host=localhost,port=9876
 * MyApp
 */
public class AevumAgent {

    public static void premain(String agentArgs, Instrumentation inst) {
        System.out.println("[Aevum] JVM agent starting...");

        // Parse agent arguments
        AgentConfig config = parseArgs(agentArgs);

        // Initialize connection to coordinator
        EventSender.initialize(config);

        // Register transformer
        inst.addTransformer(new AevumClassTransformer(config));

        System.out.println("[Aevum] JVM agent initialized (trace_id: " + config.traceId + ")");
    }

    private static AgentConfig parseArgs(String args) {
        AgentConfig config = new AgentConfig();
        if (args != null) {
            for (String arg : args.split(",")) {
                String[] parts = arg.split("=");
                if (parts.length == 2) {
                    switch (parts[0]) {
                        case "trace-id":
                            config.traceId = parts[1];
                            break;
                        case "host":
                            config.serverHost = parts[1];
                            break;
                        case "port":
                            config.serverPort = Integer.parseInt(parts[1]);
                            break;
                    }
                }
            }
        }
        return config;
    }

    static class AgentConfig {
        String traceId = "default-trace";
        String serverHost = "localhost";
        int serverPort = 9876;
    }

    static class AevumClassTransformer implements ClassFileTransformer {

        public AevumClassTransformer(AgentConfig config) {
            // Initialize EventSender with config
            EventSender.setConfig(config);
        }

        @Override
        public byte[] transform(ClassLoader loader, String className,
                Class<?> classBeingRedefined,
                ProtectionDomain protectionDomain,
                byte[] classfileBuffer) {

            // Skip JDK classes and agent classes
            if (className.startsWith("java/") ||
                    className.startsWith("javax/") ||
                    className.startsWith("sun/") ||
                    className.startsWith("io/aevum/agent/")) {
                return null;
            }

            try {
                ClassReader reader = new ClassReader(classfileBuffer);
                ClassWriter writer = new ClassWriter(reader, ClassWriter.COMPUTE_FRAMES);
                ClassVisitor visitor = new AevumClassVisitor(writer, className);
                reader.accept(visitor, ClassReader.EXPAND_FRAMES);
                return writer.toByteArray();
            } catch (Exception e) {
                System.err.println("[Aevum] Failed to instrument: " + className);
                e.printStackTrace();
                return null;
            }
        }
    }

    static class AevumClassVisitor extends ClassVisitor {
        private final String className;

        public AevumClassVisitor(ClassVisitor cv, String className) {
            super(Opcodes.ASM9, cv);
            this.className = className;
        }

        @Override
        public MethodVisitor visitMethod(int access, String name, String descriptor,
                String signature, String[] exceptions) {
            MethodVisitor mv = super.visitMethod(access, name, descriptor, signature, exceptions);
            return new AevumMethodVisitor(mv, access, name, descriptor, className);
        }
    }

    static class AevumMethodVisitor extends AdviceAdapter {
        private final String methodName;
        private final String className;

        protected AevumMethodVisitor(MethodVisitor mv, int access,
                String name, String descriptor, String className) {
            super(Opcodes.ASM9, mv, access, name, descriptor);
            this.methodName = name;
            this.className = className;
        }

        @Override
        protected void onMethodEnter() {
            // Call EventSender.recordFunctionCall(className, methodName)
            mv.visitLdcInsn(className);
            mv.visitLdcInsn(methodName);
            mv.visitMethodInsn(INVOKESTATIC,
                    "io/aevum/agent/EventSender",
                    "recordFunctionCall",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    false);
        }

        @Override
        protected void onMethodExit(int opcode) {
            // Call EventSender.recordFunctionReturn(className, methodName)
            mv.visitLdcInsn(className);
            mv.visitLdcInsn(methodName);
            mv.visitMethodInsn(INVOKESTATIC,
                    "io/aevum/agent/EventSender",
                    "recordFunctionReturn",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    false);
        }
    }

    static class EventSender {
        private static AgentConfig config;
        private static java.net.Socket socket;
        private static java.io.OutputStream output;
        private static long sequenceNumber = 0;

        public static void setConfig(AgentConfig cfg) {
            config = cfg;
        }

        public static void initialize(AgentConfig cfg) {
            config = cfg;
            try {
                socket = new java.net.Socket(config.serverHost, config.serverPort);
                output = socket.getOutputStream();
            } catch (Exception e) {
                System.err.println("[Aevum] Failed to connect to coordinator: " + e.getMessage());
            }
        }

        public static synchronized void recordFunctionCall(String className, String methodName) {
            if (output == null)
                return;

            try {
                long seq = ++sequenceNumber;
                String event = String.format(
                        "{\"event_type\":\"FunctionCall\"," +
                                "\"metadata\":{\"trace_id\":\"%s\",\"process_id\":%d," +
                                "\"thread_id\":%d,\"timestamp_ns\":%d,\"sequence_number\":%d}," +
                                "\"function_name\":\"%s\",\"module\":\"%s\"}",
                        config.traceId,
                        ProcessHandle.current().pid(),
                        Thread.currentThread().threadId(),
                        System.nanoTime(),
                        seq,
                        methodName,
                        className);

                byte[] data = event.getBytes("UTF-8");
                output.write(java.nio.ByteBuffer.allocate(4).putInt(data.length).array());
                output.write(data);
                output.flush();
            } catch (Exception e) {
                // Ignore errors to avoid disrupting application
            }
        }

        public static synchronized void recordFunctionReturn(String className, String methodName) {
            if (output == null)
                return;

            try {
                long seq = ++sequenceNumber;
                String event = String.format(
                        "{\"event_type\":\"FunctionReturn\"," +
                                "\"metadata\":{\"trace_id\":\"%s\",\"process_id\":%d," +
                                "\"thread_id\":%d,\"timestamp_ns\":%d,\"sequence_number\":%d}," +
                                "\"function_name\":\"%s\"}",
                        config.traceId,
                        ProcessHandle.current().pid(),
                        Thread.currentThread().threadId(),
                        System.nanoTime(),
                        seq,
                        methodName);

                byte[] data = event.getBytes("UTF-8");
                output.write(java.nio.ByteBuffer.allocate(4).putInt(data.length).array());
                output.write(data);
                output.flush();
            } catch (Exception e) {
                // Ignore errors
            }
        }
    }
}
