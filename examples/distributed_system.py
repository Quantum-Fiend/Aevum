"""
Example: Distributed Microservices Tracing

This example demonstrates how to trace a distributed system
with multiple services communicating over HTTP.
"""

import http.server
import socketserver
import json
import urllib.request
import threading
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'agents', 'python-agent'))

# Note: In production, import aevum_agent and attach

# ============================================
# Service A: Order Service
# ============================================

class OrderService:
    """Handles customer orders"""
    
    def __init__(self, port=8001):
        self.port = port
        self.orders = {}
        self.order_counter = 0
    
    def create_order(self, customer_id, items):
        self.order_counter += 1
        order_id = f"ORD-{self.order_counter:04d}"
        
        # Call inventory service
        inventory_ok = self._check_inventory(items)
        if not inventory_ok:
            return {"error": "Insufficient inventory"}
        
        # Call payment service
        payment_ok = self._process_payment(customer_id, items)
        if not payment_ok:
            return {"error": "Payment failed"}
        
        # Create order
        self.orders[order_id] = {
            "customer_id": customer_id,
            "items": items,
            "status": "confirmed"
        }
        
        return {"order_id": order_id, "status": "confirmed"}
    
    def _check_inventory(self, items):
        # Would call inventory service
        return True
    
    def _process_payment(self, customer_id, items):
        # Would call payment service
        return True


# ============================================
# Service B: Inventory Service
# ============================================

class InventoryService:
    """Manages product inventory"""
    
    def __init__(self, port=8002):
        self.port = port
        self.inventory = {
            "PROD-001": 100,
            "PROD-002": 50,
            "PROD-003": 200,
        }
    
    def check_availability(self, product_id, quantity):
        available = self.inventory.get(product_id, 0)
        return available >= quantity
    
    def reserve(self, product_id, quantity):
        if self.check_availability(product_id, quantity):
            self.inventory[product_id] -= quantity
            return True
        return False


# ============================================
# Service C: Payment Service
# ============================================

class PaymentService:
    """Processes payments"""
    
    def __init__(self, port=8003):
        self.port = port
        self.transactions = []
    
    def process(self, customer_id, amount):
        # Simulate payment processing
        transaction = {
            "customer_id": customer_id,
            "amount": amount,
            "status": "completed"
        }
        self.transactions.append(transaction)
        return transaction


# ============================================
# Demo
# ============================================

def run_demo():
    print("🌐 Distributed Microservices Tracing Demo")
    print("=" * 50)
    
    # In production, attach Aevum agent to each service
    # aevum_agent.attach("order-service", "localhost", 9876)
    
    # Initialize services
    order_service = OrderService()
    inventory_service = InventoryService()
    payment_service = PaymentService()
    
    print("\n📦 Creating order...")
    
    # Simulate order creation (would involve cross-service calls)
    result = order_service.create_order(
        customer_id="CUST-001",
        items=[
            {"product_id": "PROD-001", "quantity": 2},
            {"product_id": "PROD-002", "quantity": 1}
        ]
    )
    
    print(f"   Result: {result}")
    
    print("\n📊 Service states after order:")
    print(f"   Orders: {order_service.orders}")
    print(f"   Inventory: {inventory_service.inventory}")
    print(f"   Transactions: {len(payment_service.transactions)}")
    
    print("\n✅ Demo complete!")
    print("\nWith Aevum tracing, you would see:")
    print("   - Cross-service call graph")
    print("   - Causal relationships between service calls")
    print("   - Ability to replay the entire distributed transaction")
    print("   - Race condition detection if services were concurrent")


if __name__ == "__main__":
    run_demo()
