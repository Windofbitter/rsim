/// Example showing how developers would write components with the new Event-based API
#[cfg(test)]
mod examples {
    use super::super::values::traits::*;
    use super::super::values::implementations::*;
    use super::super::components::module::{EvaluationContext, ProcessorModule, PortSpec};

    /// Example 1: Simple component using convenience API (90% of use cases)
    fn simple_adder_component(ctx: &EvaluationContext, outputs: &mut EventOutputMap) -> Result<(), String> {
        // Developer just uses .get() - no need to know about Events
        let a = ctx.inputs.get::<i64>("input_a")?;
        let b = ctx.inputs.get::<i64>("input_b")?;
        
        let result = a + b;
        
        // Developer just uses .set() - Event creation is automatic
        outputs.set("output", result)?;
        
        Ok(())
    }

    /// Example 2: Timestamp-aware component for delays/scheduling
    fn delay_component(ctx: &EvaluationContext, outputs: &mut EventOutputMap) -> Result<(), String> {
        let data = ctx.inputs.get::<i64>("data")?;
        let input_timestamp = ctx.inputs.get_timestamp("data")?;
        
        // Only process data that's been in the system for at least 5 cycles
        if input_timestamp <= 5 {
            return Ok(()); // Skip output for recent data
        }
        
        outputs.set("delayed_output", data)?;
        Ok(())
    }

    /// Example 3: Advanced component with full Event access
    fn logger_component(ctx: &EvaluationContext, outputs: &mut EventOutputMap) -> Result<(), String> {
        if let Ok(event) = ctx.inputs.get_event("log_data") {
            println!("Event ID: {}, Timestamp: {}, Data: {:?}", 
                     event.event_id, 
                     event.timestamp, 
                     event.get_payload::<String>());
                     
            // Pass through the data
            let data = event.get_payload::<String>()?;
            outputs.set("logged_output", data.clone())?;
        }
        
        Ok(())
    }

    /// Example 4: Component that works with multiple input events
    fn event_correlator(ctx: &EvaluationContext, outputs: &mut EventOutputMap) -> Result<(), String> {
        // Check if we have events from multiple sources
        if ctx.inputs.has_input("sensor_a") && ctx.inputs.has_input("sensor_b") {
            let timestamp_a = ctx.inputs.get_timestamp("sensor_a")?;
            let timestamp_b = ctx.inputs.get_timestamp("sensor_b")?;
            
            // Only correlate if events are from the same cycle (synchronized)
            if timestamp_a == timestamp_b {
                let data_a = ctx.inputs.get::<f64>("sensor_a")?;
                let data_b = ctx.inputs.get::<f64>("sensor_b")?;
                
                let correlation = data_a * data_b;
                outputs.set("correlation", correlation)?;
            }
        }
        
        Ok(())
    }

    #[test]
    fn test_component_creation_examples() {
        // Developers create components the same way, just with Event-aware functions
        let _adder = ProcessorModule::new(
            "SimpleAdder",
            vec![PortSpec::input("input_a"), PortSpec::input("input_b")],
            vec![PortSpec::output("output")],
            vec![],
            simple_adder_component,
        );

        let _delay = ProcessorModule::new(
            "DelayComponent", 
            vec![PortSpec::input("data")],
            vec![PortSpec::output("delayed_output")],
            vec![],
            delay_component,
        );

        let _logger = ProcessorModule::new(
            "LoggerComponent",
            vec![PortSpec::input("log_data")],
            vec![PortSpec::output("logged_output")],
            vec![],
            logger_component,
        );

        let _correlator = ProcessorModule::new(
            "EventCorrelator",
            vec![PortSpec::input("sensor_a"), PortSpec::input("sensor_b")],
            vec![PortSpec::output("correlation")],
            vec![],
            event_correlator,
        );
    }
}